"""Minimal MUGEN SFF v1/v2 reader that decodes sprites to RGBA PIL Images."""
import io
import struct

from PIL import Image


class Sprite:
    def __init__(self, group, item, axis, w=0, h=0):
        self.group, self.item, self.axis = group, item, axis
        self.w, self.h = w, h
        self.data = None
        self.fmt = None
        self.pal_index = 0
        self.pal = None
        self.same_pal = False


def _pcx_decode(buf):
    xmin, ymin, xmax, ymax = struct.unpack_from("<4H", buf, 4)
    planes, bpl = buf[65], struct.unpack_from("<H", buf, 66)[0]
    w, h = xmax - xmin + 1, ymax - ymin + 1
    if planes != 1:
        raise ValueError("unsupported PCX planes=%d" % planes)
    out = bytearray(bpl * h)
    i, j, n = 128, 0, len(out)
    while j < n and i < len(buf):
        b = buf[i]
        i += 1
        if b >= 0xC0:
            cnt, v = b & 0x3F, buf[i]
            i += 1
            out[j:j + cnt] = bytes([v]) * min(cnt, n - j)
            j += cnt
        else:
            out[j] = b
            j += 1
    if bpl == w:
        px = bytes(out)
    else:
        px = b"".join(bytes(out[r * bpl:r * bpl + w]) for r in range(h))
    pal = buf[-768:] if len(buf) > 769 and buf[-769] == 12 else None
    return (w, h, px), pal


def _rle8(src, n):
    out, i = bytearray(), 0
    while len(out) < n and i < len(src):
        b = src[i]
        i += 1
        if b & 0xC0 == 0x40:
            out += bytes([src[i]]) * (b & 0x3F)
            i += 1
        else:
            out.append(b)
    return out


def _rle5(src, n):
    out, i = bytearray(), 0
    while len(out) < n and i < len(src):
        rl = src[i]
        i += 1
        dl = src[i] & 0x7F
        c = 0
        if src[i] >> 7:
            i += 1
            c = src[i]
        i += 1
        while True:
            out.append(c)
            rl -= 1
            if rl < 0:
                dl -= 1
                if dl < 0:
                    break
                c, rl = src[i] & 0x1F, src[i] >> 5
                i += 1
    return out


def _lz5(src, n):
    p = bytearray(n)
    last = len(src) - 1
    i = j = 0
    ct, cts, rb, rbc = src[0], 0, 0, 0
    i = min(i + 1, last)
    while j < n:
        d = src[i]
        i = min(i + 1, last)
        if ct & (1 << cts):
            if d & 0x3F == 0:
                d = ((d << 2) | src[i]) + 1
                i = min(i + 1, last)
                cnt = src[i] + 2
                i = min(i + 1, last)
            else:
                rb |= (d & 0xC0) >> rbc
                rbc += 2
                cnt = d & 0x3F
                if rbc < 8:
                    d = src[i] + 1
                    i = min(i + 1, last)
                else:
                    d, rb, rbc = rb + 1, 0, 0
            while cnt >= 0:
                if j < n:
                    p[j] = p[j - d]
                    j += 1
                cnt -= 1
        else:
            if d & 0xE0 == 0:
                cnt = src[i] + 8
                i = min(i + 1, last)
            else:
                cnt, d = d >> 5, d & 0x1F
            while cnt > 0 and j < n:
                p[j] = d
                j += 1
                cnt -= 1
        cts += 1
        if cts >= 8:
            ct, cts = src[i], 0
            i = min(i + 1, last)
    return p


class SFF:
    def __init__(self, raw):
        if raw[:11] != b"ElecbyteSpr":
            raise ValueError("not an SFF file")
        self.version = raw[15]
        self.sprites, self.palettes = [], []
        (self._read_v1 if self.version == 1 else self._read_v2)(raw)
        self.index = {}
        for s in self.sprites:
            self.index.setdefault((s.group, s.item), s)

    def _read_v1(self, raw):
        nimg, off = struct.unpack_from("<I", raw, 20)[0], struct.unpack_from("<I", raw, 24)[0]
        prev_pal = None
        for _ in range(nimg):
            if off + 32 > len(raw):
                break
            nxt, ln, ax, ay, g, it, link, same = struct.unpack_from("<IIhhHHHB", raw, off)
            s = Sprite(g, it, (ax, ay))
            s.same_pal = bool(same)
            if ln == 0 and link < len(self.sprites):
                src = self.sprites[link]
                s.data, s.pal = src.data, src.pal
            else:
                s.data = raw[off + 32: off + 32 + ln]
                own = _pcx_decode_pal(s.data)
                s.pal = prev_pal if (same and prev_pal) else (own or prev_pal)
            prev_pal = s.pal
            self.sprites.append(s)
            if nxt == 0:
                break
            off = nxt

    def _read_v2(self, raw):
        spr_off, nspr, pal_off, npal, ld_off, _, td_off = struct.unpack_from("<7I", raw, 36)
        for k in range(npal):
            _, _, ncol, link, o, ln = struct.unpack_from("<4HII", raw, pal_off + 16 * k)
            if ln == 0 and link < len(self.palettes):
                self.palettes.append(self.palettes[link])
                continue
            rgba = raw[ld_off + o: ld_off + o + ncol * 4]
            pal = bytearray(768)
            for c in range(min(ncol, 256)):
                pal[c * 3:c * 3 + 3] = rgba[c * 4:c * 4 + 3]
            self.palettes.append(bytes(pal))
        for k in range(nspr):
            g, it, w, h, ax, ay, link, fmt, _, o, ln, pi, fl = struct.unpack_from(
                "<4H2hHBBIIHH", raw, spr_off + 28 * k)
            s = Sprite(g, it, (ax, ay), w, h)
            if ln == 0 and link < len(self.sprites):
                src = self.sprites[link]
                s.data, s.fmt, s.w, s.h, s.pal_index = src.data, src.fmt, src.w, src.h, src.pal_index
            else:
                base = td_off if fl & 1 else ld_off
                s.data, s.fmt, s.pal_index = raw[base + o: base + o + ln], fmt, pi
            self.sprites.append(s)

    def rgba(self, sprite, palette=None):
        """Decode a sprite to an RGBA PIL Image; index 0 is transparent."""
        if self.version == 1:
            (w, h, px), own = _pcx_decode(sprite.data)
            return _apply(w, h, px, palette or sprite.pal or own)
        n, src = sprite.w * sprite.h, sprite.data[4:]
        if sprite.fmt in (10, 11, 12):
            img = Image.open(io.BytesIO(src))
            if sprite.fmt == 10:
                pal = palette or self.palettes[sprite.pal_index]
                return _apply(img.width, img.height, img.tobytes(), pal)
            return img.convert("RGBA")
        dec = {0: lambda b, m: b[:m], 2: _rle8, 3: _rle5, 4: _lz5}[sprite.fmt]
        buf = bytes(dec(sprite.data if sprite.fmt == 0 else src, n))
        buf = buf.ljust(n, b"\0")[:n]
        return _apply(sprite.w, sprite.h, buf, palette or self.palettes[sprite.pal_index])


def _pcx_decode_pal(buf):
    return buf[-768:] if len(buf) > 769 and buf[-769] == 12 else None


def _apply(w, h, px, pal):
    img = Image.frombytes("P", (w, h), px)
    raw_pal = bytes(pal).ljust(768, b"\0")[:768] if pal else bytes(768)
    img.putpalette(raw_pal)
    rgba = img.convert("RGBA")
    alpha = Image.frombytes("L", (w, h), px.translate(_ALPHA_LUT))
    rgba.putalpha(alpha)
    return rgba


_ALPHA_LUT = bytes([0] + [255] * 255)


def read_act(raw):
    """ACT files store colors in reverse order (index 255 first)."""
    padded = raw[:768].ljust(768, b"\0")
    return b"".join(padded[i:i + 3] for i in range(765, -1, -3))
