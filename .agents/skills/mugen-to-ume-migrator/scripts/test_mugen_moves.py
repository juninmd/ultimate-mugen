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

    def test_command_names_match_without_regard_to_case(self):
        states = [ctrl("changestate", value="1000", triggerall='command = "special 1"', trigger1="statetype != a")]
        self.assertEqual(routes(states, {"SPECIAL 1": ["~D, DF, F, a"]}), [(1000, "special 1", "~D, DF, F, a", "stand", 0)])

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

    def test_explod_and_envshake_become_vfx_and_quake(self):
        actions = {300: {"frames": FRAMES, "loop": None}, 7000: {"frames": FRAMES, "loop": None}}
        states = {300: {"def": {"anim": "300"}, "ctrl": [
            dict(ctrl("explod", anim="7000", pos="10, -40", removetime="20", bindtime="1", scale="1.5, 1.5", trigger1="AnimElem = 1"), at=[1]),
            dict(ctrl("explod", anim="F0", trigger1="AnimElem = 1"), at=[1]),
            dict(ctrl("envshake", time="12", ampl="-3", trigger1="AnimElem = 2"), at=[2]),
        ]}}
        move, wanted = build_move(300, states, actions, "burst", "x", "stand", 0)
        self.assertEqual(move["vfx"], [{"at": 1, "anim": "a7000", "offset": [10.0, -40.0], "time": 20, "follow": True, "scale": 1.5}])
        self.assertEqual(move["quake"], [{"at": 2, "time": 12, "amp": 3.0}])
        self.assertIn(7000, wanted)


def move_anim_number(move):
    return 900000 + int(move["anim"][3:])


if __name__ == "__main__":
    unittest.main()
