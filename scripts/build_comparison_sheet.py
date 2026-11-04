import os
from PIL import Image, ImageDraw, ImageFont

def make_comparison():
    out_dir = "proto_sprites"
    os.makedirs(out_dir, exist_ok=True)

    # 1680 x 1200 high-res presentation board
    W, H = 1680, 1200
    canvas = Image.new("RGBA", (W, H), (22, 25, 34, 255))
    draw = ImageDraw.Draw(canvas)

    # Fonts
    font_title = ImageFont.truetype("C:/Windows/Fonts/arialbd.ttf", 36)
    font_subtitle = ImageFont.truetype("C:/Windows/Fonts/arial.ttf", 18)
    font_section = ImageFont.truetype("C:/Windows/Fonts/arialbd.ttf", 22)
    font_badge = ImageFont.truetype("C:/Windows/Fonts/arialbd.ttf", 14)
    font_label = ImageFont.truetype("C:/Windows/Fonts/arialbd.ttf", 16)
    font_desc = ImageFont.truetype("C:/Windows/Fonts/arial.ttf", 13)

    # Header gradient bar / background
    draw.rectangle([(0, 0), (W, 110)], fill=(30, 34, 48, 255))
    draw.line([(0, 110), (W, 110)], fill=(55, 65, 88, 255), width=2)

    # Title & Subtitle
    draw.text((40, 24), "ULTIMATE MUGEN — PROTÓTIPO DE REDESIGN PIXEL ART", fill=(255, 255, 255), font=font_title)
    draw.text((40, 72), "Personagem: Kaito Arashi · Comparação Direta: Vetor Procedural (Atual) vs Pixel Art 32-Bit Capcom/SNK (Novo via Gemini)", fill=(130, 200, 230), font=font_subtitle)

    # Status tag
    tag_text = "PROTÓTIPO APROVADO"
    draw.rounded_rectangle([(W - 250, 36), (W - 40, 76)], radius=6, fill=(35, 140, 80))
    draw.text((W - 232, 46), tag_text, fill=(255, 255, 255), font=font_badge)

    # Poses metadata
    poses = [
        ("01. Idle Stance", "before_01_idle.png", "after_01_idle_transparent.png", "Guarda base combate"),
        ("02. Snap Jab", "before_02_jab.png", "after_02_jab_transparent.png", "Soco direto rápido"),
        ("03. Roundhouse Kick", "before_03_kick.png", "after_03_kick_transparent.png", "Chute alto circular"),
        ("04. Rising Tempest", "before_04_uppercut.png", "after_04_uppercut_transparent.png", "Shoryuken ascendente"),
        ("05. Gale Blast", "before_05_blast.png", "after_05_blast_transparent.png", "Palma de choque vento"),
        ("06. Victory Taunt", "before_06_victory.png", "after_06_victory_transparent.png", "Comemoração de vitória"),
    ]

    col_w = 260
    start_x = 40
    y_row1 = 170
    y_row2 = 560
    card_h = 340

    # Section Headers
    # Row 1: ANTES (Vetor Atual)
    draw.rounded_rectangle([(start_x, 125), (start_x + 360, 155)], radius=4, fill=(50, 58, 76))
    draw.text((start_x + 12, 130), "ANTES: SKELETAL RIG PROCEDURAL (VETORIAL)", fill=(240, 160, 160), font=font_badge)

    # Row 2: DEPOIS (Pixel Art Novo)
    draw.rounded_rectangle([(start_x, 515), (start_x + 380, 545)], radius=4, fill=(30, 70, 95))
    draw.text((start_x + 12, 520), "DEPOIS: PIXEL ART RETRO ARCADE (CAPCOM / SNK STYLE)", fill=(100, 235, 210), font=font_badge)

    ground_y1 = y_row1 + card_h - 25
    ground_y2 = y_row2 + card_h - 25

    for idx, (title, before_fn, after_fn, desc) in enumerate(poses):
        cx = start_x + idx * (col_w + 10)

        # Card backdrops
        draw.rounded_rectangle([(cx, y_row1), (cx + col_w, y_row1 + card_h)], radius=8, fill=(28, 32, 44), outline=(42, 48, 64), width=1)
        draw.rounded_rectangle([(cx, y_row2), (cx + col_w, y_row2 + card_h)], radius=8, fill=(25, 33, 44), outline=(36, 68, 88), width=1)

        # Ground line indicator
        draw.line([(cx + 15, ground_y1), (cx + col_w - 15, ground_y1)], fill=(55, 62, 80), width=2)
        draw.line([(cx + 15, ground_y2), (cx + col_w - 15, ground_y2)], fill=(45, 80, 100), width=2)

        # Pose labels
        draw.text((cx + 12, y_row1 + 10), title, fill=(210, 215, 230), font=font_label)
        draw.text((cx + 12, y_row1 + 30), desc, fill=(120, 130, 155), font=font_desc)

        draw.text((cx + 12, y_row2 + 10), title, fill=(130, 225, 215), font=font_label)
        draw.text((cx + 12, y_row2 + 30), desc, fill=(110, 150, 160), font=font_desc)

        # Load and paste Before Sprite
        before_p = os.path.join(out_dir, before_fn)
        if os.path.exists(before_p):
            b_img = Image.open(before_p).convert("RGBA")
            # Fit in max height ~260, anchor at bottom
            scale_b = min(250.0 / b_img.height, (col_w - 20) / b_img.width, 1.0)
            target_w = int(b_img.width * scale_b)
            target_h = int(b_img.height * scale_b)
            b_resized = b_img.resize((target_w, target_h), Image.Resampling.LANCZOS)
            px = cx + (col_w - target_w) // 2
            py = ground_y1 - target_h
            canvas.alpha_composite(b_resized, (px, py))

        # Load and paste After Sprite
        after_p = os.path.join(out_dir, after_fn)
        if os.path.exists(after_p):
            a_img = Image.open(after_p).convert("RGBA")
            # Fit in max height ~260, anchor at bottom
            scale_a = min(250.0 / a_img.height, (col_w - 20) / a_img.width, 1.0)
            target_w = int(a_img.width * scale_a)
            target_h = int(a_img.height * scale_a)
            a_resized = a_img.resize((target_w, target_h), Image.Resampling.NEAREST)
            px = cx + (col_w - target_w) // 2
            py = ground_y2 - target_h
            canvas.alpha_composite(a_resized, (px, py))

    # Bottom summary and architectural analysis box
    y_info = 930
    draw.rounded_rectangle([(start_x, y_info), (W - start_x, H - 35)], radius=10, fill=(26, 32, 46), outline=(48, 70, 95), width=1)

    # Info title
    draw.text((start_x + 25, y_info + 18), "ANÁLISE COMPARATIVA E DIRETRIZES TÉCNICAS", fill=(255, 215, 110), font=font_section)

    features = [
        ("Contorno & Definição:", "Substitui traços vetoriais lisos por contornos pretos pixel-a-pixel autênticos de 1px com seletividade arcade."),
        ("Paleta de Cores:", "Fidelidade mantida ao design original (Kimono Verde-Água #2f8f9d, Faixa Amarela #f2b233, Faixas nos punhos e pés descalços)."),
        ("Sombreado & Volume:", "Elimina o efeito 'marionete/boneco flash', introduzindo cel shading arcade manual clássico com 3 níveis de sombra."),
        ("Integração Engine:", "A estrutura UMF suporta mapeamento de sprites bitmap indexados via 'art.sprites' preservando caixas de colisão (hit/hurt)."),
    ]

    for i, (head, text) in enumerate(features):
        line_y = y_info + 60 + i * 36
        draw.text((start_x + 25, line_y), "• " + head, fill=(100, 220, 240), font=font_badge)
        draw.text((start_x + 225, line_y), text, fill=(200, 210, 225), font=font_subtitle)

    # 3x Zoom Face comparison on the bottom right
    zoom_w, zoom_h = 240, 160
    zx = W - start_x - zoom_w - 20
    zy = y_info + 50
    draw.rounded_rectangle([(zx - 8, zy - 8), (zx + zoom_w + 8, zy + zoom_h + 8)], radius=6, fill=(18, 22, 32), outline=(50, 75, 100), width=1)

    # Crop idle head
    idle_after = Image.open(os.path.join(out_dir, "after_01_idle_transparent.png")).convert("RGBA")
    head_crop = idle_after.crop((50, 0, 140, 90))
    head_zoomed = head_crop.resize((zoom_w, zoom_h), Image.Resampling.NEAREST)
    canvas.alpha_composite(head_zoomed, (zx, zy))
    draw.text((zx, zy - 26), "ZOOM PIXEL ART 2.5X (DETALHE FACIAL)", fill=(240, 200, 100), font=font_desc)

    out_file = os.path.join(out_dir, "kaito_before_after_comparison.png")
    canvas.save(out_file)
    print("Saved comparison board to:", out_file)

if __name__ == "__main__":
    make_comparison()
