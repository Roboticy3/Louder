class_name CharacterAvatar extends Node3D

@export var character_data:CharacterData
@export var body:CharacterBody3D
@export var navigation_agent:NavigationAgent3D
@export var walk_speed := 3.0

@export var group := &"character_avatar"

func _ready() -> void:
	add_to_group(group)
	
	navigation_agent.target_reached.connect(_on_target_reached)

var navigation_input := Vector3.ZERO
func _physics_process(delta: float) -> void:
	var next_position := navigation_agent.get_next_path_position()
	var current_position := body.global_position
	var error := next_position - current_position
	navigation_input = error.normalized() * walk_speed
	
	if error.length() > 0.1:
		body.velocity = navigation_input
		
		var angle_error := body.global_basis.z.signed_angle_to(navigation_input, body.global_basis.y)
		if absf(angle_error) > 0.1:
			body.rotation.y += 3 * signf(angle_error) * delta
		else:
			body.rotation.y += angle_error * delta
		DebugDraw3d.draw_arrow(body.global_position, -body.global_basis.z, Color.BLUE)
	else:
		body.velocity = Vector3.ZERO
	
	body.velocity += Vector3.DOWN * delta
	
	body.move_and_slide()
	
	DebugDraw3d.draw_box(navigation_agent.target_position, Quaternion.IDENTITY, Vector3.ONE, Color.BLACK)
	DebugDraw3d.draw_box(next_position, Quaternion.IDENTITY, Vector3.ONE, Color.RED)

@rpc("call_local", "any_peer", "reliable")
func navigate_to_position_rpc(to:Vector3):
	if !multiplayer.is_server():
		return
	var remote_id := multiplayer.get_remote_sender_id()
	var remote_data := Network.get_character_of(remote_id)
	if remote_data == character_data:
		navigate_to_position.rpc(to)

@rpc("call_local", "authority", "reliable")
func navigate_to_position(to:Vector3):
	navigation_agent.target_position = to

@rpc("call_local", "any_peer", "reliable")
func navigate_to_interactable_rpc(interactable_path:NodePath):
	if !multiplayer.is_server():
		return

	var remote_id := multiplayer.get_remote_sender_id()
	var remote_data := Network.get_character_of(remote_id)
	if remote_data == character_data:
		navigate_to_interactable.rpc(interactable_path)

var intent_to_interact:InteractableObject
@rpc("call_local", "authority", "reliable")
func navigate_to_interactable(interactable_path:NodePath):
	var i = get_node_or_null(interactable_path)
	if !(i is InteractableObject):
		return
	
	navigate_to_position(i.get_interact_position())
	intent_to_interact = i

func _on_target_reached():
	if intent_to_interact:
		#Start dialogue. Connected controller should see this and trigger dialogue UI
		print("starting dialogue with ", intent_to_interact)
	
	intent_to_interact = null
	
