import os
from PIL import Image, ImageDraw, ImageFont

def make_gallery():
    out_dir = "proto_sprites"
    os.makedirs(out_dir, exist_ok=True)

    chars = [
        ("kaito", "Kaito Arashi", "Tempest Karate", "assets/chars/sprites/kaito/01_idle.png"),
        ("shion", "Shion Kurogane", "Moon Blade Exiled", "assets/chars/sprites/shion/01_idle.png"),
        ("akio", "Akio Hoshi", "Starlight Striker", "assets/chars/sprites/akio/01_idle.png"),
        ("suzume", "Suzume Kagerou", "Crimson Lotus Kunoichi", "assets/chars/sprites/suzume/01_idle.png"),
        ("bruno", "Bruno Volk", "Heavyweight Bulldozer", "assets/chars/sprites/bruno/01_idle.png"),
        ("sylene", "Sylene Vashti", "Glacial Sovereign", "assets/chars/sprites/sylene/01_idle.png"),
        ("volta", "VX-9 Volta", "Neon Cyber Android", "assets/chars/sprites/volta/01_idle.png"),
        ("lian", "Lian Feng", "Jade Lantern Wushu", "assets/chars/sprites/lian/01_idle.png"),
        ("garan", "Garan Ashen Oni", "Nether Flame Demon", "assets/chars/sprites/garan/01_idle.png"),
        ("marlow", "Marlow Reyes", "Sky & Sea Corsair", "assets/chars/sprites/marlow/01_idle.png"),
        ("kaname", "Kaname Fuyutsuki", "Frostmoon Iaido", "assets/chars/sprites/kaname/01_idle.png"),
        ("jin", "Jin Kazaguruma", "Gale Shadow Shinobi", "assets/chars/sprites/jin/01_idle.png"),
        ("daichi", "Daichi Ginrou", "Golden Beast Hermit", "assets/chars/sprites/daichi/01_idle.png"),
        ("valdric", "Valdric Morn", "Crimson Warlord", "assets/chars/sprites/valdric/01_idle.png"),
        ("haruki", "Haruki Sumi", "Runic Scholar", "assets/chars/sprites/haruki/01_idle.png"),
        ("oboro", "Mikazuki Oboro", "Eclipse Illusionist [Boss]", "assets/chars/sprites/oboro/01_idle.png"),
    ]

    # 4 columns x 4 rows
    cols = 4
    rows = 4
    card_w = 280
    card_h = 360
    pad_x = 30
    pad_y = 30
    header_h = 110

    W = cols * card_w + (cols + 1) * pad_x
    H = header_h + rows * card_h + (rows + 1) * pad_y

    canvas = Image.new("RGBA", (W, H), (20, 24, 34, 255))
    draw = ImageDraw.Draw(canvas)

    font_title = ImageFont.truetype("C:/Windows/Fonts/arialbd.ttf", 34)
    font_sub = ImageFont.truetype("C:/Windows/Fonts/arial.ttf", 17)
    font_name = ImageFont.truetype("C:/Windows/Fonts/arialbd.ttf", 18)
    font_archetype = ImageFont.truetype("C:/Windows/Fonts/arial.ttf", 13)
    font_badge = ImageFont.truetype("C:/Windows/Fonts/arialbd.ttf", 13)
    font_pending = ImageFont.truetype("C:/Windows/Fonts/arial.ttf", 14)

    # Header
    draw.rectangle([(0, 0), (W, header_h)], fill=(28, 33, 46, 255))
    draw.line([(0, header_h), (W, header_h)], fill=(50, 62, 85), width=2)
    draw.text((40, 25), "ULTIMATE MUGEN — NATIVE ROSTER PIXEL ART REDESIGN", fill=(255, 255, 255), font=font_title)
    draw.text((40, 70), "Personagens Nativos Recriados em 32-Bit Arcade Pixel Art (Capcom CPS2 / Neo-Geo SNK via Gemini AI)", fill=(110, 210, 235), font=font_sub)

    completed_count = sum(1 for _, _, _, p in chars if os.path.exists(p))
    tag = f"{completed_count}/16 PERSONAGENS IMPLEMENTADOS"
    draw.rounded_rectangle([(W - 320, 35), (W - 40, 75)], radius=6, fill=(40, 130, 80))
    draw.text((W - 305, 46), tag, fill=(255, 255, 255), font=font_badge)

    for i, (cid, name, archetype, path) in enumerate(chars):
        c = i % cols
        r = i // cols
        x = pad_x + c * (card_w + pad_x)
        y = header_h + pad_y + r * (card_h + pad_y)

        # Card container
        has_sprite = os.path.exists(path)
        border_col = (42, 52, 72) if has_sprite else (35, 40, 52)
        card_bg = (26, 30, 42) if has_sprite else (22, 25, 34)
        draw.rounded_rectangle([(x, y), (x + card_w, y + card_h)], radius=8, fill=card_bg, outline=border_col, width=1)
        ground_y = y + card_h - 22
        draw.line([(x + 20, ground_y), (x + card_w - 20, ground_y)], fill=(55, 68, 92) if has_sprite else (35, 42, 56), width=2)

        # Name & archetype header in card
        name_col = (255, 255, 255) if has_sprite else (140, 150, 165)
        arch_col = (120, 180, 210) if has_sprite else (80, 100, 120)
        draw.text((x + 16, y + 14), name, fill=name_col, font=font_name)
        draw.text((x + 16, y + 36), archetype, fill=arch_col, font=font_archetype)

        if has_sprite:
            sprite = Image.open(path).convert("RGBA")
            max_h = 250.0
            scale = min(max_h / sprite.height, (card_w - 30) / sprite.width, 1.0)
            tw = int(sprite.width * scale)
            th = int(sprite.height * scale)
            scaled = sprite.resize((tw, th), Image.Resampling.NEAREST)

            px = x + (card_w - tw) // 2
            py = ground_y - th
            canvas.alpha_composite(scaled, (px, py))
        else:
            # Placeholder text
            draw.text((x + 70, y + 160), "[ AGUARDANDO COTA ]", fill=(90, 105, 125), font=font_pending)
            draw.text((x + 85, y + 185), "Reset às 04:49 UTC", fill=(65, 80, 95), font=font_archetype)

    out_file = os.path.join(out_dir, "native_roster_pixel_gallery.png")
    canvas.save(out_file)
    print("Saved gallery to:", out_file)

if __name__ == "__main__":
    make_gallery()
