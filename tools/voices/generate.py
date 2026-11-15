import json, os, struct, subprocess, sys, tempfile
import numpy as np
import soundfile as sf
from kokoro_onnx import Kokoro

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..")
CHARS = os.path.join(ROOT, "assets", "chars")
OUT = os.path.join(ROOT, "assets", "voices", "voices.bank")
MODEL = os.environ.get("KOKORO_MODEL", "kokoro-v1.0.onnx")
VOICES = os.environ.get("KOKORO_VOICES", "voices-v1.0.bin")

CAST = {
    "kaito": dict(voice="am_adam", speed=1.05),
    "shion": dict(voice="bm_lewis", speed=0.95),
    "akio": dict(voice="am_puck", speed=1.1),
    "suzume": dict(voice="af_nova", speed=1.05),
    "bruno": dict(voice="am_onyx", speed=0.9, fx="asetrate=16000*0.93,aresample=16000"),
    "sylene": dict(voice="bf_isabella", speed=0.95, fx="aecho=0.8:0.7:40:0.25"),
    "volta": dict(voice="am_echo", speed=1.0, fx="tremolo=f=70:d=0.35,aecho=0.8:0.8:12:0.4"),
    "lian": dict(voice="af_kore", speed=1.0),
    "garan": dict(voice="bm_fable", speed=0.85, fx="asetrate=16000*0.82,aresample=16000,aecho=0.8:0.8:60:0.3"),
    "marlow": dict(voice="bm_george", speed=0.95),
    "kaname": dict(voice="af_river", speed=0.95),
    "jin": dict(voice="am_eric", speed=1.1),
    "daichi": dict(voice="am_fenrir", speed=0.9),
    "valdric": dict(voice="bm_daniel", speed=0.88, fx="asetrate=16000*0.9,aresample=16000,aecho=0.8:0.7:50:0.25"),
    "haruki": dict(voice="am_liam", speed=1.0),
    "oboro": dict(voice="pm_alex", speed=0.95, lang="pt-br", fx="aecho=0.8:0.7:70:0.3"),
}
SHOUT = ("special", "super", "finisher")


def shouts(doc):
    found = {}
    for m in doc.get("moves", []):
        if m["kind"] in SHOUT and not m["id"].startswith("ex_"):
            found[m["id"]] = m["name"]
    for form in doc.get("abilities", {}).get("forms", []):
        for m in form.get("moves", []):
            if m["kind"] in SHOUT and not m["id"].startswith("ex_"):
                found.setdefault(m["id"], m["name"])
    return found


def trim(x, rate):
    peak = float(np.max(np.abs(x))) or 1.0
    keep = np.where(np.abs(x) > peak * 0.02)[0]
    if len(keep):
        x = x[max(0, keep[0] - int(rate * 0.02)): keep[-1] + int(rate * 0.12)]
    return x


RATE = 16000
GAP = int(RATE * 0.05)


def master(samples, rate, fx):
    with tempfile.TemporaryDirectory() as d:
        src, dst = os.path.join(d, "a.wav"), os.path.join(d, "b.wav")
        sf.write(src, samples, rate)
        chain = ["aresample=%d" % RATE]
        if fx:
            chain.append(fx)
        chain += ["highpass=f=70", "afade=t=in:d=0.005", "afade=t=out:st=%f:d=0.08" % max(0.0, len(samples) / rate - 0.08), "loudnorm=I=-16:TP=-1.5:LRA=7"]
        subprocess.run(["ffmpeg", "-v", "error", "-y", "-i", src, "-af", ",".join(chain), "-ac", "1", "-ar", str(RATE), dst], check=True)
        data, _ = sf.read(dst)
        return data.astype(np.float32)


def stream(clips):
    parts, spans, at = [], [], 0
    for move_id, data in clips:
        spans.append((move_id, at, len(data)))
        parts += [data, np.zeros(GAP, np.float32)]
        at += len(data) + GAP
    with tempfile.TemporaryDirectory() as d:
        wav, ogg = os.path.join(d, "a.wav"), os.path.join(d, "a.ogg")
        sf.write(wav, np.concatenate(parts), RATE)
        subprocess.run(["ffmpeg", "-v", "error", "-y", "-i", wav, "-c:a", "libvorbis", "-b:a", "24k", ogg], check=True)
        return spans, open(ogg, "rb").read()


def main():
    only = sys.argv[1:]
    tts = Kokoro(MODEL, VOICES)
    banks = []
    for char, cfg in CAST.items():
        if only and char not in only:
            continue
        doc = json.load(open(os.path.join(CHARS, char + ".json"), encoding="utf-8"))
        clips = []
        for move_id, name in shouts(doc).items():
            text = name.replace(":", ",") + "!"
            samples, rate = tts.create(text, voice=cfg["voice"], speed=cfg["speed"], lang=cfg.get("lang", "en-us" if cfg["voice"][0] == "a" else "en-gb"))
            clips.append((move_id, master(trim(samples, rate), rate, cfg.get("fx"))))
        spans, ogg = stream(clips)
        banks.append((char, spans, ogg))
        print(char, len(spans), len(ogg), flush=True)
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "wb") as f:
        f.write(b"UVB2" + struct.pack("<II", RATE, len(banks)))
        for char, spans, ogg in banks:
            name = char.encode()
            f.write(struct.pack("<B", len(name)) + name + struct.pack("<H", len(spans)))
            for move_id, start, count in spans:
                mid = move_id.encode()
                f.write(struct.pack("<B", len(mid)) + mid + struct.pack("<II", start, count))
            f.write(struct.pack("<I", len(ogg)) + ogg)
    print("total bytes", sum(len(b[2]) for b in banks))


main()
