extends Node

const ClientProperties := ClientDataLayer.ClientProperties

@export var config:PSConfig
@export var controller:PSController
@export var group := &"merchant_dialogue_avatar"

func _ready():
	add_to_group(group)
	
	update_controller_speakers()
	ClientDataLayer.data_updated.connect(func (_a, _b):
		update_controller_speakers()
	)

func update_controller_speakers():
	if !controller.simulator:
		controller.find_simulator()
	
	if !is_inside_tree() or !is_faux_authority():
		return
	
	var current_controller := ControllerEventLayer.get_connected_controller()
	var character: String = ClientDataLayer.get_local_data().get(ClientProperties.character, "")
	if !character.is_empty() and controller != current_controller:
		controller.speak_as = [character]
		var relationships := config.get_relationships(character)
		controller.listen_to = relationships
		ControllerEventLayer.request_controller.rpc_id(1, controller.get_path())
	elif character.is_empty() and controller == current_controller:
		ControllerEventLayer.disconnect_controller(controller)

func get_faux_authority():
	return int(name)

func is_faux_authority():
	return get_faux_authority() == multiplayer.get_unique_id()
