extends Node

const ClientProperties := ClientDataLayer.ClientProperties

@export var config:PSConfig
@export var controller:PSController
@export var group := &"merchant_dialogue_avatar"

#eventually want to replace this with more intentional selection of simulators based on user input in the world
@export var simulator_group := &"ps_simulator"

func _ready():
	add_to_group(group)
	
	update_controller_speakers()
	ClientDataLayer.data_updated.connect(func (_a, _b):
		update_controller_speakers()
	)

func update_controller_speakers():
	if !is_inside_tree() or !is_faux_authority():
		return
	
	var character: String = ClientDataLayer.get_local_data().get(ClientProperties.character, "")
	if !character.is_empty():
		controller.simulator = get_tree().get_first_node_in_group(simulator_group)
		controller.speak_as = [character]
		var relationships := config.get_relationships(character)
		controller.listen_to = relationships
		ControllerEventLayer.connect_controller(controller.get_path())
	else:
		ControllerEventLayer.disconnect_controller(controller.get_path())

func get_faux_authority():
	return int(name)

func is_faux_authority():
	return get_faux_authority() == multiplayer.get_unique_id()
