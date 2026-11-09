from tools.spritekit.collision_boxes import anatomical_boxes, attack_boxes

CLSN1_FRONT_SPECIAL = 0.85
CLSN1_FRONT_NORMAL = 0.7
SPECIAL_KINDS = ("special", "air_special", "super", "projectile")


def grow_attack_box(boxes, height, kind):
    if not boxes or height <= 0:
        return boxes
    x0 = min(b[0] for b in boxes)
    y0 = min(b[1] for b in boxes)
    x1 = max(b[2] for b in boxes)
    y1 = max(b[3] for b in boxes)
    if x1 <= 0:
        return boxes
    special = kind in SPECIAL_KINDS
    base = (CLSN1_FRONT_SPECIAL if special else CLSN1_FRONT_NORMAL) * height
    cap = 0.5 if special else 0.35
    new_x1 = int(round(min(max(x1 + 0.12 * height, base), x1 + cap * height)))
    minimum = 0.6 * height
    if y1 - y0 < minimum:
        center = (y0 + y1) / 2
        y0, y1 = center - minimum / 2, center + minimum / 2
        if y1 > 0:
            y0, y1 = y0 - y1, 0
    return [(min(x0, 0), int(round(y0)), new_x1, int(round(y1)))]


def box_lines(number, boxes):
    if not boxes:
        return []
    return [f"Clsn{number}: {len(boxes)}"] + [f"  Clsn{number}[{i}] = {x0},{y0},{x1},{y1}" for i, (x0, y0, x1, y1) in enumerate(boxes)]


def build_action(move, library, height=None):
    number = int(move.get("anim", move["state"]))
    kind = move.get("kind", "normal")
    loop = int(move.get("loop", 0))
    lines = [f"[Begin Action {number}]"]
    for index, frame in enumerate(move["frames"]):
        group, item, ticks = int(frame[0]), int(frame[1]), int(frame[2])
        flags = [x for x in frame[3:] if isinstance(x, str)]
        options = next((x for x in frame[3:] if isinstance(x, dict)), {})
        if loop and index == loop:
            lines.append("Loopstart")
        found = library.mask_and_axis(group, item) if group >= 0 else None
        body = options.get("clsn2")
        if body is None:
            body = anatomical_boxes(found[0], found[1], zones=2) if found else []
        attack = options.get("clsn1")
        if attack is None:
            attack = attack_boxes(found[0], found[1]) if found and "hit" in flags else []
            if attack and height:
                attack = grow_attack_box(attack, height, kind)
        lines += box_lines(2, body) + box_lines(1, attack)
        line = f"{group},{item}, {options.get('x', 0)},{options.get('y', 0)}, {ticks}"
        if options.get("flip") or options.get("trans"):
            line += f", {options.get('flip', '')}, {options.get('trans', '')}"
        lines.append(line)
    return number, "\n".join(lines) + "\n"


def hit_frames(action_text):
    frames = []
    index = 0
    for line in action_text.splitlines():
        stripped = line.strip()
        if stripped.startswith("Clsn1:"):
            frames.append(index + 1)
        elif stripped and not stripped.lower().startswith(("clsn", "[", "loopstart")):
            index += 1
    return frames
