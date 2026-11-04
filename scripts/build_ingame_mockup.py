import os
from PIL import Image, ImageDraw, ImageFont, ImageOps

def make_ingame_mockup():
    out_dir = "proto_sprites"
    os.makedirs(out_dir, exist_ok=True)

    stage = Image.open("screenshots/stages/S-dojo-1-calm.jpg").convert("RGBA")
    hud = Image.open("screenshots/50-hud-v3-top.jpg").convert("RGBA")
    W, H = stage.size # 1280, 720

    # Shion idle (facing left -> flip horizontal)
    shion_img = Image.open("proto_sprites/shion_idle.png").convert("RGBA")
    shion_flipped = ImageOps.mirror(shion_img)

    # Scale fighters to match in-game proportions (approx height 220px)
    fighter_h = 220
    floor_y = 636 # Dojo floor ground line

    # 1) Scale Vector Kaito
    kaito_vector = Image.open("proto_sprites/before_01_idle.png").convert("RGBA")
    scale_kv = fighter_h / kaito_vector.height
    w_kv = int(kaito_vector.width * scale_kv)
    kv_scaled = kaito_vector.resize((w_kv, fighter_h), Image.Resampling.LANCZOS)

    # 2) Scale Pixel Art Kaito
    kaito_pixel = Image.open("proto_sprites/after_01_idle_transparent.png").convert("RGBA")
    scale_kp = fighter_h / kaito_pixel.height
    w_kp = int(kaito_pixel.width * scale_kp)
    kp_scaled = kaito_pixel.resize((w_kp, fighter_h), Image.Resampling.NEAREST)

    # 3) Scale Shion
    scale_s = fighter_h / shion_flipped.height
    w_s = int(shion_flipped.width * scale_s)
    s_scaled = shion_flipped.resize((w_s, fighter_h), Image.Resampling.LANCZOS)

    # Fighter positions:
    # Player 1 (Kaito): x = 380
    # Player 2 (Shion): x = 760
    p1_x = 380
    p2_x = 760

    # Draw contact shadow function
    def draw_shadow(img_canvas, x, y, width):
        shadow = Image.new("RGBA", (width + 40, 24), (0, 0, 0, 0))
        sdraw = ImageDraw.Draw(shadow)
        sdraw.ellipse([(0, 2), (width + 36, 20)], fill=(0, 0, 0, 110))
        img_canvas.alpha_composite(shadow, (x - 20, y - 8))

    # Helper to construct a match frame
    def render_match(p1_sprite, is_pixel=False):
        frame = stage.copy()
        
        # Shadows
        draw_shadow(frame, p1_x, floor_y, p1_sprite.width)
        draw_shadow(frame, p2_x, floor_y, s_scaled.width)

        # Place fighters
        frame.alpha_composite(p1_sprite, (p1_x, floor_y - p1_sprite.height))
        frame.alpha_composite(s_scaled, (p2_x, floor_y - s_scaled.height))

        # Place HUD at top
        frame.alpha_composite(hud, (0, 0))

        # Bottom super meter bar mockups
        bdraw = ImageDraw.Draw(frame)
        # P1 meter
        bdraw.rectangle([(60, 680), (320, 700)], fill=(20, 24, 34, 230), outline=(60, 70, 90), width=1)
        bdraw.rectangle([(62, 682), (240, 698)], fill=(47, 143, 157, 255))
        # P2 meter
        bdraw.rectangle([(W - 320, 680), (W - 60, 700)], fill=(20, 24, 34, 230), outline=(60, 70, 90), width=1)
        bdraw.rectangle([(W - 240, 682), (W - 62, 698)], fill=(220, 50, 70, 255))

        return frame

    screen_before = render_match(kv_scaled, is_pixel=False)
    screen_after = render_match(kp_scaled, is_pixel=True)

    # Assemble into a vertical comparison board (1280 x 1540)
    canvas = Image.new("RGBA", (W, 1540), (18, 22, 32, 255))
    draw = ImageDraw.Draw(canvas)

    font_title = ImageFont.truetype("C:/Windows/Fonts/arialbd.ttf", 26)
    font_badge = ImageFont.truetype("C:/Windows/Fonts/arialbd.ttf", 16)
    font_sub = ImageFont.truetype("C:/Windows/Fonts/arial.ttf", 15)

    # Header
    draw.rectangle([(0, 0), (W, 60)], fill=(28, 34, 48, 255))
    draw.text((30, 16), "ULTIMATE MUGEN — IN-GAME BATTLE PROTOTYPE (1280x720 HD)", fill=(255, 255, 255), font=font_title)

    # Panel 1: Top (y = 60..780)
    canvas.paste(screen_before, (0, 60))
    draw.rounded_rectangle([(30, 80), (360, 122)], radius=6, fill=(180, 45, 60, 230))
    draw.text((45, 92), "ANTES: VECTOR RIG PROCEDURAL", fill=(255, 255, 255), font=font_badge)

    # Divider bar
    draw.rectangle([(0, 780), (W, 800)], fill=(15, 18, 25, 255))
    draw.line([(0, 790), (W, 790)], fill=(45, 65, 90), width=2)

    # Panel 2: Bottom (y = 800..1520)
    canvas.paste(screen_after, (0, 800))
    draw.rounded_rectangle([(30, 820), (430, 862)], radius=6, fill=(25, 140, 110, 230))
    draw.text((45, 832), "DEPOIS: 32-BIT ARCADE PIXEL ART (GEMINI)", fill=(255, 255, 255), font=font_badge)

    out_file = os.path.join(out_dir, "kaito_ingame_mockup.png")
    canvas.save(out_file)
    print("Saved in-game mockup to:", out_file)

if __name__ == "__main__":
    make_ingame_mockup()
