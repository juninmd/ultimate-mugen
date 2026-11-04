[Command]
name = "a"
command = a
time = 1

[Statedef -1]

[State -1, attack]
type = ChangeState
value = 200
triggerall = Command = "a"
trigger1 = StateType = S && Ctrl
