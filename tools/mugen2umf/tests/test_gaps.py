import os
import tempfile
import unittest

from mugen2umf.chain import repeat_elems
from mugen2umf.charge import charge_gain, charge_spec
from mugen2umf.palettes import exact_palettes
from mugen2umf.throw import throw_steps


def ctrl(kind, **body):
    pairs = [(k, v) for k, v in body.items()]
    return {"type": kind, "body": dict(pairs), "pairs": pairs}


def state(anim, *ctrls, **extra):
    return {"def": {"anim": str(anim), **extra}, "ctrl": list(ctrls)}


def act(colors):
    raw = bytearray(768)
    for index, color in colors.items():
        raw[(255 - index) * 3:(255 - index) * 3 + 3] = bytes(color)
    return bytes(raw)


class Palettes(unittest.TestCase):
    def test_only_the_indices_that_change_are_mapped_and_index_zero_is_ignored(self):
        base = act({0: (1, 1, 1), 1: (10, 20, 30), 2: (50, 60, 70)})
        alt = act({0: (9, 9, 9), 1: (200, 0, 0), 2: (50, 60, 70)})
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "2.act")
            open(path, "wb").write(alt)
            self.assertEqual(exact_palettes(base, [path]), [{"exact": {"#0a141e": "#c80000"}}])

    def test_an_identical_palette_keeps_its_slot_so_palette_numbers_stay_aligned(self):
        base = act({1: (10, 20, 30)})
        alt = act({1: (1, 2, 3)})
        with tempfile.TemporaryDirectory() as d:
            same, other = os.path.join(d, "2.act"), os.path.join(d, "3.act")
            open(same, "wb").write(base)
            open(other, "wb").write(alt)
            self.assertEqual(exact_palettes(base, [same, other, os.path.join(d, "missing.act")]), [{"exact": {}}, {"exact": {"#0a141e": "#010203"}}])

    def test_no_difference_anywhere_means_no_palettes(self):
        base = act({1: (10, 20, 30)})
        with tempfile.TemporaryDirectory() as d:
            same = os.path.join(d, "2.act")
            open(same, "wb").write(base)
            self.assertEqual(exact_palettes(base, [same]), [])


class Charge(unittest.TestCase):
    def loop(self):
        return {735: {"frames": [{"t": 4}] * 6, "loop": 0}}

    def test_gain_is_the_power_added_per_animation_cycle_divided_by_its_length(self):
        st = state(735, ctrl("poweradd", trigger1="Time = [0,12]", value="10"), ctrl("changestate", trigger1="AnimTime = 0", value="735"))
        self.assertEqual(charge_gain(st, self.loop()), {"start": 5.42, "growth": 0, "max": 5.42})

    def test_a_state_that_does_not_cycle_has_no_constant_gain(self):
        st = state(735, ctrl("poweradd", trigger1="Time = [0,12]", value="10"))
        self.assertIsNone(charge_gain(st, self.loop()))

    def test_a_power_trigger_that_is_not_a_time_window_is_not_guessed(self):
        st = state(735, ctrl("poweradd", trigger1="Life > 100", value="10"), ctrl("changestate", trigger1="AnimTime = 0", value="735"))
        self.assertIsNone(charge_gain(st, self.loop()))

    def test_the_full_power_state_that_only_plays_the_end_anim_belongs_to_the_ability(self):
        states = {
            735: state(735, ctrl("changestate", trigger1='command != "charge"', value="740"), ctrl("changestate", trigger1="Power >= 3000", value="741")),
            740: state(740, ctrl("changestate", trigger1="AnimTime = 0", value="0")),
            741: state(740, ctrl("changestate", trigger1="AnimTime = 0", value="0")),
        }
        actions = {735: {"frames": [{"t": 4}], "loop": 0}, 740: {"frames": [{"t": 4}], "loop": None}}
        _, _, consumed = charge_spec(735, states, actions, lambda n: f"a{n}")
        self.assertEqual(consumed, {735, 740, 741})


class Throws(unittest.TestCase):
    def test_binds_sound_damage_and_launch_become_steps(self):
        p1 = state(810,
                   ctrl("targetbind", trigger1="AnimElem = 1, >= 0", pos="10,0"),
                   ctrl("targetbind", trigger1="AnimElem = 2, >= 0", pos="20,-5"),
                   ctrl("playsnd", trigger1="AnimElem = 3", value="3,0"),
                   ctrl("targetlifeadd", trigger1="animtime = 0", value="-60"),
                   ctrl("targetstate", trigger1="AnimElem = 4", value="821"))
        victim = state(5050, ctrl("lifeadd", trigger1="Time = 0", value="-40"), velset="3.5,-8")
        steps, total = throw_steps(p1, {821: victim})
        self.assertEqual(total, 100)
        self.assertEqual(steps, [{"at": 2, "bind": [20.0, -5.0]}, {"at": 3, "sound": [3, 0]}, {"at": 4, "damage": 100, "launch": [3.5, -8.0]}])

    def test_without_a_release_state_nothing_is_invented(self):
        p1 = state(810, ctrl("targetbind", trigger1="AnimElem = 1", pos="10,0"))
        self.assertIsNone(throw_steps(p1, {}))


class Loops(unittest.TestCase):
    def test_elements_after_the_loop_start_repeat_with_every_pass(self):
        action = {"frames": [{"t": 1}] * 4, "loop": 1}
        self.assertEqual(repeat_elems([2], action, 10), [2, 5, 8])
        self.assertEqual(repeat_elems([1], action, 10), [1])

    def test_a_segment_that_is_not_repeated_keeps_its_elements(self):
        self.assertEqual(repeat_elems([2], {"frames": [{"t": 1}] * 4, "loop": 1}, 4), [2])


if __name__ == "__main__":
    unittest.main()
