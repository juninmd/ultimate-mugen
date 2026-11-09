from .animation import hit_frames
from .controller import as_list, controller, pair
from .hitdef import hit_controllers, hit_elements
from .kinds import AIR_KINDS


def state_header(move, number, anim):
    kind = move.get("kind", "normal")
    air_move = kind in AIR_KINDS
    crouch = kind == "crouch"
    helper = bool(move.get("helper"))
    stance = move.get("type", "A" if air_move else "C" if crouch else "S")
    physics = move.get("physics", "N" if helper else "A" if air_move else "C" if crouch else "S")
    lines = [f"[Statedef {number}]", f"type = {stance}", "movetype = A", f"physics = {physics}",
             "juggle = 1", "ctrl = 0", f"anim = {anim}", f"sprpriority = {move.get('sprpriority', 2)}"]
    if not air_move and not helper:
        lines.append("facep2 = 1")
    if "velset" in move:
        lines.append(f"velset = {pair(move['velset'])}")
    if move.get("power") and "_phase_of" not in move:
        lines.append(f"poweradd = {-abs(int(move['power']))}")
    return "\n".join(lines) + "\n\n"


def super_pause(move, defaults):
    spec = move.get("superpause") if isinstance(move.get("superpause"), dict) else {}
    return controller("SuperPause", "super pause", trigger1="!time", time=int(spec.get("time", 30)),
                      movetime=spec.get("movetime", 0), anim=spec.get("anim", -1),
                      sound=spec.get("sound", defaults.get("super_sound", -1)),
                      darken=int(spec.get("darken", 1)), unhittable=1, poweradd=0, p2defmul=1)


def effect_controllers(move):
    out = ""
    for vfx in move.get("vfx", []):
        out += controller("Explod", vfx.get("label", "vfx"),
                          trigger1=vfx.get("trigger", f"animelem = {vfx['elem']}"), anim=vfx["anim"],
                          id=vfx.get("id", vfx["anim"]), pos=pair(vfx.get("pos", [0, 0])),
                          postype=vfx.get("postype", "p1"), facing=vfx.get("facing", 1),
                          bindtime=vfx.get("bind", -1), removetime=vfx.get("removetime", -2),
                          scale=pair(vfx.get("scale", [1, 1])), sprpriority=vfx.get("sprpriority", 3),
                          ownpal=1, removeongethit=1, supermovetime=999, pausemovetime=999,
                          ignorehitpause=1, trans=vfx.get("trans"))
    for helper in move.get("helpers", []):
        out += controller("Helper", helper.get("label", "helper"),
                          trigger1=helper.get("trigger") or (f"animelem = {helper['elem']}" if "elem" in helper else "!time"),
                          stateno=helper["state"], id=helper.get("id", helper["state"]),
                          pos=pair(helper.get("pos", [0, 0])), postype=helper.get("postype", "p1"),
                          facing=helper.get("facing"), ownpal=1, persistent=0)
    return out


def cancel_controllers(move):
    out = ""
    for cancel in move.get("cancel", []):
        if isinstance(cancel, str):
            command, state = cancel.split(":")[:2]
            cancel = {"command": command, "state": state}
        condition = f" && ({cancel['condition']})" if cancel.get("condition") else ""
        out += controller("ChangeState", f"cancel into {cancel['state']}", value=cancel["state"],
                          triggerall=f"(roundstate = 2) && ({cancel.get('contact', 'movecontact')})",
                          trigger1=f'(command = "{cancel["command"]}"){condition}')
    return out


def ending_controllers(move):
    kind = move.get("kind", "normal")
    air_move = kind in AIR_KINDS
    if move.get("_next") is not None:
        out = ""
        if air_move:
            out += controller("ChangeState", "land", trigger1="(pos y >= 0) && (vel y > 0)", value=move.get("land", 52))
        return out + controller("ChangeState", "next phase", trigger1="!animtime", value=move["_next"])
    if move.get("helper"):
        return controller("DestroySelf", "end", trigger1="!animtime")
    end = move.get("end", 50 if air_move else 11 if kind == "crouch" else 0)
    out = ""
    if air_move:
        out += controller("ChangeState", "land", trigger1="(pos y >= 0) && (vel y > 0)", value=move.get("land", 52))
        return out + controller("ChangeState", "end", trigger1="!animtime", value=end, ctrl=1)
    return out + controller("ChangeState", "end", trigger1="!animtime",
                            value=f"cond(pos y < 0, 50, {end})", ctrl=1)


def build_state(move, defaults, action_text=""):
    number = int(move["state"])
    anim = int(move.get("anim", number))
    kind = move.get("kind", "normal")
    out = state_header(move, number, anim)
    if (kind == "super" and "_phase_of" not in move) or move.get("superpause"):
        out += super_pause(move, defaults)
    if move.get("voice"):
        out += controller("PlaySnd", "voice", trigger1="!time", value=move["voice"], channel=defaults.get("voice_channel", 0))
    for sound in move.get("sounds", []):
        out += controller("PlaySnd", "sound", trigger1=f"animelem = {sound['elem']}", value=sound["value"])
    for vel in move.get("vel", []):
        out += controller(vel.get("type", "VelSet"), "velocity",
                          trigger1=vel.get("trigger", f"animelem = {vel['elem']}"), x=vel.get("x"), y=vel.get("y"))
    frames = hit_frames(action_text)
    for hit in as_list(move.get("hit", [])):
        for element in hit_elements(hit, frames):
            out += hit_controllers(hit, element, move, defaults)
    out += effect_controllers(move) + cancel_controllers(move)
    if move.get("extra"):
        out += move["extra"].strip() + "\n\n"
    return out + ending_controllers(move)
