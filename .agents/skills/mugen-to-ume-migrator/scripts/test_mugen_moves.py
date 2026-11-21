import unittest

from PIL import Image

from mugen_parse import contain_axis
from mugen_moves import build_move, convert_input, frame_at, routes


def ctrl(kind, **body):
    pairs = [(k, v) for k, v in body.items()]
    return {"type": kind, "body": dict(pairs), "pairs": pairs}


FRAMES = [{"t": 3, "clsn1": []}, {"t": 4, "clsn1": [[0, -60, 30, -40]]}, {"t": 5, "clsn1": []}]


class Moves(unittest.TestCase):
    def test_inputs_keep_motion_and_join_hold_chords(self):
        self.assertEqual(convert_input("~D, DF, F, x"), "~D, DF, F, x")
        self.assertEqual(convert_input("/F, a"), "F+a")

    def test_time_trigger_maps_to_the_running_frame(self):
        self.assertEqual(frame_at(ctrl("playsnd", trigger1="Time = 4"), FRAMES), [2])
        self.assertEqual(frame_at(ctrl("hitdef", trigger1="AnimElem = 3"), FRAMES), [3])

    def test_routes_skip_locked_and_hold_only_commands(self):
        cmds = {"qcf_x": ["~D, DF, F, x"], "charge": ["/$x"]}
        states = [
            ctrl("changestate", value="1000", triggerall='command = "qcf_x"', trigger1="statetype = A"),
            ctrl("changestate", value="1100", trigger1='command = "qcf_x" && ctrl = 0'),
            ctrl("changestate", value="1200", trigger1='command = "charge"'),
        ]
        self.assertEqual(routes(states, cmds), [(1000, "qcf_x", "~D, DF, F, x", "air", 0)])

    def test_unconditional_state_chain_becomes_one_sequence(self):
        actions = {300: {"frames": FRAMES, "loop": None}, 301: {"frames": FRAMES, "loop": None}}
        states = {
            300: {"def": {"anim": "300"}, "ctrl": [ctrl("hitdef", trigger1="AnimElem = 2", damage="40"), ctrl("changestate", value="301", trigger1="AnimTime = 0")]},
            301: {"def": {"anim": "301"}, "ctrl": [ctrl("hitdef", trigger1="AnimElem = 2", damage="60", superpause="1"), ctrl("superpause", trigger1="Time = 0")]},
        }
        move, _ = build_move(300, states, actions, "combo", "x", "stand", 0)
        self.assertEqual([h["at"] for h in move["hits"]], [2, 5])
        self.assertEqual(move["kind"], "super")
        self.assertEqual(len(actions[move_anim_number(move)]["frames"]), 6)

    def test_axis_outside_the_sprite_is_padded_in(self):
        image, axis = contain_axis(Image.new("RGBA", (40, 8)), (-10, 51))
        self.assertEqual((image.size, axis), ((50, 51), (0, 51)))


def move_anim_number(move):
    return 900000 + int(move["anim"][3:])


if __name__ == "__main__":
    unittest.main()
