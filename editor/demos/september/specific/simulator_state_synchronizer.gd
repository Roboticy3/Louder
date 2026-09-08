extends Node

@export var state_listener_group := &"state_listener"

func _ready():
	multiplayer.peer_connected.connect(_on_peer_connected)

func _on_peer_connected(id:int):
	if !multiplayer.is_server():
		return
	
	if current_simulator and current_simulator.state:
		var names := current_simulator.state.scope.get_names()
		for n in names:
			var v = current_simulator.state.scope.get(n)
			sync_state_changed.rpc_id(id, current_simulator.get_path(), n, v)

var current_simulator:PSSimulator
func _on_state_changed_server(simulator:PSSimulator, variable:String, value) -> void:
	if !multiplayer.is_server():
		return
	
	if simulator != current_simulator:
		current_simulator = simulator
	
	#print("syncing simulator state from server: ", variable)
	get_tree().call_group(state_listener_group, &"_on_state_changed", simulator, variable, value)
	sync_state_changed.rpc(simulator.get_path(), variable, value)

@rpc("authority", "call_remote", "reliable")
func sync_state_changed(simulator_path:NodePath, variable:String, value) -> void:
	var node := get_node_or_null(simulator_path)
	if !(node is PSSimulator):
		return
	
	var simulator:PSSimulator = node
	if !simulator.state:
		push_error("Dialogue state missing from ", simulator, "! Probable desync.")
		simulator.state = PSDialogueState.new()
	
	var current_value = simulator.state.scope.get(variable)
	if current_value == value: return
	
	#print("recieving synchronized simulator state from server: ", variable)
	simulator.state.scope.set(variable, value)
	get_tree().call_group(state_listener_group, &"_on_state_changed", simulator, variable, value)
	simulator.state_variable_changed.emit(simulator, variable, value)
