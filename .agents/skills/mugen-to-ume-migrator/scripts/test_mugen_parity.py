import os
import tempfile
import unittest

from mugen_chain import segment
from mugen_charge import charge_spec
from mugen_motion import motion
from mugen_parse import read_states
from mugen_routes import routes
from mugen_scripts import passthrough


def ctrl(kind, **body):
    pairs = [(k, v) for k, v in body.items()]
    return {"type": kind, "body": dict(pairs), "pairs": pairs}


def state(anim, *ctrls, **extra):
    return {"def": {"anim": str(anim), **extra}, "ctrl": list(ctrls)}


class Parity(unittest.TestCase):
    def test_duplicate_statedef_keeps_the_first_block_only(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "a.cns")
            open(path, "w").write("[Statedef 7825]\nanim = 7500\n[State 7825, a]\ntype = PosAdd\nx = 125\n"
                                  "[Statedef 7825]\nanim = 7502\n[State 7825, b]\ntype = PosAdd\nx = 125\n")
            states = read_states([path])
        self.assertEqual(states[7825]["def"]["anim"], "7500")
        self.assertEqual(len(states[7825]["ctrl"]), 1, "a second Statedef with the same number must not double its controllers")

    def test_timed_exit_repeats_a_looping_animation_until_the_original_time(self):
        action = {"frames": [{"t": 5}, {"t": 5}], "loop": 0}
        st = state(735, ctrl("changestate", value="3188", trigger1="Time = 140"))
        frames = segment(st, action, 3188)
        self.assertEqual(sum(f["t"] for f in frames), 140)

    def test_flight_only_commands_become_fly_moves_and_ground_normals_survive(self):
        cmds = {"a": ["a"]}
        minus1 = [
            ctrl("changestate", value="300", triggerall='command = "a"', trigger1="StateNo = 3001", trigger2="stateno = 250"),
            ctrl("changestate", value="200", triggerall='command = "a"', trigger1="statetype = S && ctrl"),
        ]
        self.assertEqual([(v, w) for v, _, _, w, _ in routes(minus1, cmds, [3001])], [(300, "fly"), (200, "stand")])

    def test_charge_keeps_the_original_start_and_end_animations_and_sounds(self):
        actions = {730: {"frames": []}, 735: {"frames": []}, 740: {"frames": []}}
        states = {
            730: state(730, ctrl("changestate", value="735", trigger1="AnimTime = 0"), ctrl("playsnd", value="7, 0", trigger1="AnimElem = 1")),
            735: state(735, ctrl("changestate", value="740", trigger1='command != "charge"'), ctrl("poweradd", value="10", trigger1="1"),
                       ctrl("playsnd", value="1, 0", trigger1="AnimElem = 3"), ctrl("playsnd", value="1, 0", trigger1="AnimElem = 10")),
            740: state(740),
        }
        spec, _, used = charge_spec(735, states, actions, lambda n: f"a{n}")
        self.assertEqual((spec["startAnim"], spec["endAnim"], spec["startSound"], spec["soundAt"]), ("a730", "a740", [7, 0], [3, 10]))
        self.assertEqual(used, {730, 735, 740})

    def test_intro_states_pass_through_with_animation_references(self):
        actions = {190: {"frames": []}, 199: {"frames": []}}
        states = {-1: {"def": {}, "ctrl": []}, 190: state(190, ctrl("explod", anim="199", trigger1="AnimElem = 1"), ctrl("changestate", value="191", trigger1="Time = 0"))}
        spec, used = passthrough(states, {}, actions, lambda n: {190: "intro"}.get(n, f"a{n}"), set())
        self.assertIn("anim = @anim(intro)", spec["cns"])
        self.assertIn("anim = @anim(a199)", spec["cns"])
        self.assertEqual(sorted(used), [190, 199])

    def test_taunt_on_start_moves_off_the_charge_button(self):
        states = {-1: {"def": {}, "ctrl": [ctrl("changestate", value="195", trigger1='command = "start"')]}, 195: state(195)}
        spec, _ = passthrough(states, {"start": ["s"]}, {195: {"frames": []}}, lambda n: "taunt", set())
        self.assertEqual(spec["commands"], [{"name": "umf_start", "input": "x+y", "display": "Taunt"}])
        self.assertIn('command = "umf_start"', spec["cns"])

    def test_motion_controllers_map_to_umf_fields_and_conditional_ones_are_reported(self):
        def c(kind, at, **body):
            return dict(ctrl(kind, **body), at=at, span=(5, 12))
        fields, dropped = motion([
            c("velmul", [], trigger1="Time >= 0", x=".93"),
            c("veladd", [], trigger1="Time >= 0", y=".56"),
            c("posfreeze", [], trigger1="time >= 0"),
            c("posset", [5], trigger1="Time = 0", y="0"),
            c("palfx", [6], trigger1="Time = 0", time="20", add="30,30,30", sinadd="64,64,64,64"),
            c("posset", [], trigger1="Pos Y < -175", y="-175"),
        ])
        self.assertEqual(fields["velMul"], [{"from": 5, "to": 12, "x": 0.93}])
        self.assertEqual(fields["velAdd"], [{"from": 5, "to": 12, "y": 0.56}], "a per-tick VelAdd is the move's own gravity")
        self.assertEqual(fields["posFreeze"], [{"from": 5, "to": 12}])
        self.assertEqual(fields["posSet"], [{"at": 5, "y": 0.0}])
        self.assertEqual(fields["palFx"], [{"at": 6, "time": 20, "add": [30, 30, 30], "sinAdd": [64, 64, 64, 64]}])
        self.assertEqual(dropped, ["posset"], "a conditional PosSet has no frame to land on, so it must be reported, not guessed")


if __name__ == "__main__":
    unittest.main()
