import os
import subprocess
import json
import time

CHARS_DIR = "/home/antigravity/mugen/chars"
REPORT_MD = "docs/evidence/mugen-100-smoke-test/REPORT.md"
REPORT_JSON = "docs/evidence/mugen-100-smoke-test/report.json"
BIN_PATH = "target/release/examples/mugen_check"

def find_characters():
    results = []
    for entry in sorted(os.listdir(CHARS_DIR)):
        full_dir = os.path.join(CHARS_DIR, entry)
        if not os.path.isdir(full_dir):
            continue
        for root, _, files in os.walk(full_dir):
            for f in files:
                if f.lower().endswith(".def"):
                    p = os.path.join(root, f)
                    try:
                        with open(p, "rb") as fp:
                            c = fp.read().lower()
                            if b"[files]" in c and b"cns" in c:
                                results.append((entry, root, f))
                                break
                    except:
                        pass
            else:
                continue
            break
    return results

def run_smoke_test():
    chars = find_characters()
    print(f"Discovered {len(chars)} characters to test.")
    
    os.makedirs("docs/evidence/mugen-100-smoke-test", exist_ok=True)
    
    records = []
    total_passed = 0
    total_failed = 0
    
    start_all = time.time()
    for idx, (folder_name, char_path, def_file) in enumerate(chars):
        t0 = time.time()
        print(f"[{idx+1}/{len(chars)}] Testing {folder_name}...", end=" ", flush=True)
        
        proc = subprocess.run(
            [BIN_PATH, char_path],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            timeout=60
        )
        
        elapsed = time.time() - t0
        output = proc.stdout + proc.stderr
        
        record = {
            "index": idx + 1,
            "folder": folder_name,
            "def": def_file,
            "name": folder_name,
            "author": "Unknown",
            "anims": 0,
            "states": 0,
            "sprites": 0,
            "load_ms": 0,
            "hits": 0,
            "distinct_states": 0,
            "round": 0,
            "wins": [],
            "status": "FAIL",
            "diagnostics": "",
            "raw": output
        }
        
        for line in output.splitlines():
            line_s = line.strip()
            if line_s.startswith("DIAG"):
                record["diagnostics"] = line_s
            elif line_s.startswith("OK"):
                record["status"] = "OK"
                parts = line_s.split("|")
                for part in parts:
                    part = part.strip()
                    if part.startswith("name="):
                        for item in part.split():
                            if item.startswith("name="):
                                record["name"] = item.split("=", 1)[1].strip('"')
                            elif item.startswith("author="):
                                record["author"] = item.split("=", 1)[1].strip('"')
                            elif item.startswith("anims="):
                                record["anims"] = int(item.split("=")[1])
                            elif item.startswith("states="):
                                record["states"] = int(item.split("=")[1])
                            elif item.startswith("sprites="):
                                record["sprites"] = int(item.split("=")[1])
                            elif item.startswith("load="):
                                record["load_ms"] = int(item.split("=")[1].replace("ms", ""))
                    elif "60s CPU fight:" in part:
                        sub = part.replace("60s CPU fight:", "").strip()
                        for item in sub.split():
                            if item.startswith("hits="):
                                record["hits"] = int(item.split("=")[1])
                            elif item.startswith("distinct_states="):
                                record["distinct_states"] = int(item.split("=")[1])
                            elif item.startswith("round="):
                                record["round"] = int(item.split("=")[1])
            elif line_s.startswith("PANIC"):
                record["status"] = "PANIC"
        
        if record["status"] == "OK":
            total_passed += 1
            print(f"OK ({record['load_ms']}ms, {record['sprites']} spr, {record['hits']} hits, {record['distinct_states']} states)")
        else:
            total_failed += 1
            print(f"{record['status']}")
            
        records.append(record)

    total_time = time.time() - start_all
    print(f"\nCompleted in {total_time:.2f}s: {total_passed} PASSED, {total_failed} FAILED out of {len(chars)} chars.")

    with open(REPORT_JSON, "w") as fp:
        json.dump(records, fp, indent=2)

    with open(REPORT_MD, "w") as fp:
        fp.write("# Relatório de Compatibilidade & Smoke Test Individual (100+ Personagens M.U.G.E.N)\n\n")
        fp.write(f"- **Total de Personagens Testados**: {len(records)}\n")
        fp.write(f"- **Sucessos**: {total_passed} ({total_passed/len(records)*100:.1f}%)\n")
        fp.write(f"- **Falhas/Incompatíveis**: {total_failed}\n")
        fp.write(f"- **Tempo Total de Execução**: {total_time:.2f}s\n")
        fp.write(f"- **Metodologia**: Cada personagem foi isolado, validado quanto à sintaxe de scripts, integridade da folha de sprites (.sff), animações (.air), tabela de comandos (.cmd) e submetido a uma simulação de combate CPU vs CPU em tempo real de 60 segundos (3.600 ticks a 60 Hz).\n\n")
        
        fp.write("## Tabela Detalhada de Resultados\n\n")
        fp.write("| # | Lutador | Autor | Sprites | Animações | Estados | Carga | Golpes (Hits) | Estados Ativos | Status |\n")
        fp.write("|---|---|---|---|---|---|---|---|---|---|\n")
        for r in records:
            name_display = r["name"] if r["name"] else r["folder"]
            author_display = r["author"] if r["author"] else "-"
            status_badge = "✅ PASS" if r["status"] == "OK" else ("⚠️ PANIC" if r["status"] == "PANIC" else "❌ FAIL")
            fp.write(f"| {r['index']} | **{name_display}** | {author_display} | {r['sprites']} | {r['anims']} | {r['states']} | {r['load_ms']}ms | {r['hits']} | {r['distinct_states']} | {status_badge} |\n")

if __name__ == "__main__":
    run_smoke_test()
