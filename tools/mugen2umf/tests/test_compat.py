import unittest

from mugen2umf.hitdefs import hit_sound
from mugen2umf.moves import _fits
from mugen2umf.parse import sprite_image
from mugen2umf.scripts import _renumber_triggers, helper_closure, passthrough


def ctrl(kind, **body):
    pairs = [(k, v) for k, v in body.items()]
    return {"type": kind, "body": dict(pairs), "pairs": pairs}


class Sprite:
    def __init__(self, pal, same=False, root=None):
        self.pal, self.same_pal, self.axis = pal, same, (0, 0)
        self.root = root or self


class FakeSff:
    def __init__(self, sprites, legacy):
        self.sprites, self.legacy_shared = sprites, legacy
        self.index = {(0, k): s for k, s in enumerate(sprites)}

    def rgba(self, sprite, palette):
        return palette


class Compat(unittest.TestCase):
    def test_own_sound_hitsound_keeps_the_fighter_bank(self):
        self.assertEqual(hit_sound({"hitsound": "S172,13"}), {"hitSound": [172, 13], "ownSound": True})
        self.assertEqual(hit_sound({"hitsound": "5,2"}), {"hitSound": [5, 2]}, "a plain hitsound is the common fight bank")

    def test_air_only_controllers_are_left_out_of_ground_moves(self):
        air = ctrl("helper", trigger1="AnimElem = 8 && StateType = A", stateno="21100")
        ground = ctrl("helper", trigger1="AnimElem = 8 && StateType != A", stateno="21100")
        either = ctrl("playsnd", trigger1="StateType = A", trigger2="Time = 0", value="1,0")
        self.assertFalse(_fits(air, "stand"))
        self.assertTrue(_fits(ground, "stand"))
        self.assertFalse(_fits(ground, "air"))
        self.assertTrue(_fits(either, "stand"), "another trigger group can still fire it")

    def test_parent_state_checks_follow_the_native_move_and_enemy_checks_do_not(self):
        text = "Parent,Stateno = 1100 && enemynear,stateno = 1100 && p2stateno = 1100 && stateno = 21100"
        out = _renumber_triggers(text, {21100: 70005}, {1100: "qcb_x"})
        self.assertEqual(out, "Parent,Stateno = @move(qcb_x) && enemynear,stateno = 1100 && p2stateno = 1100 && stateno = 70005")

    def test_hitdef_custom_states_join_the_helper_closure(self):
        states = {21100: {"ctrl": [ctrl("hitdef", p2stateno="21160"), ctrl("hitoverride", stateno="21101")]},
                  21160: {"ctrl": []}, 21101: {"ctrl": []}}
        self.assertEqual(helper_closure([21100], states), {21100: 70000, 21160: 70001, 21101: 70002})

    def test_global_states_pass_through_with_move_references(self):
        states = {-3: {"def": {}, "ctrl": [ctrl("playsnd", trigger1="stateno = 1102", value="8,1")]}}
        spec, _ = passthrough(states, {}, {}, str, set(), {}, {1102: "qcb_a"})
        self.assertIn("[Statedef -3]", spec["cns"])
        self.assertIn("trigger1 = stateno = @move(qcb_a)", spec["cns"])

    def test_shared_palette_follows_the_previous_sprite_on_mugen_1(self):
        act, own = bytes([1]) * 768, bytes([2]) * 768
        first, effect = Sprite(act), Sprite(own)
        shared = Sprite(None, True, effect)
        new = FakeSff([first, effect, shared], legacy=False)
        old = FakeSff([first, effect, shared], legacy=True)
        self.assertIsNone(sprite_image(new, act, 0, 2)[0], "a MUGEN 1.x effect keeps its own palette")
        self.assertEqual(sprite_image(old, act, 0, 2)[0], act, "WinMUGEN shared sprites take the character palette")


if __name__ == "__main__":
    unittest.main()
