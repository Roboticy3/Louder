extends Node

@export var state_listener_group := &"state_listener"

func _on_state_changed_server(simulator:PSSimulator, variable:String, value) -> void:
	if !multiplayer.is_server():
		return
	
	sync_state_changed.rpc(simulator.get_path(), variable, value)

func sync_state_changed(simulator_path:NodePath, variable:String, value) -> void:
	var node := get_node_or_null(simulator_path)
	if !(node is PSSimulator):
		return
	
	var simulator:PSSimulator = node
	if !simulator.state:
		push_error("Dialogue state missing from ", simulator, "! Probable desync.")
		simulator.state = PSDialogueState.new()
	
	var current_value = simulator.state.scope.get(variable)
	if current_value != value:
		get_tree().call_group(state_listener_group, &"_on_state_changed", simulator, variable, value)
	simulator.state.scope.set(variable, value)
