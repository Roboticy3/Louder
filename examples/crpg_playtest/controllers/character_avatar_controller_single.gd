class_name CharacterAvatarControllerSingle extends Node

## Control a single narrative character, representing them in dialogue and connecting to the appropriate avatar in
## the game world if available.

@export var group := &"character_controller"

# The group of character avatars.
@export var character_avatar_group := &"character_avatar"

@export var character_data:CharacterData :
	set(new_character_data):
		if new_character_data != character_data:
			character_data = new_character_data
			locate_character()

@export var camera_rig:CameraRigSimple 
@export var scrolling_dialogue:ScrollingDialogue

@export_flags_3d_physics var raycast_collision_flags := 0b11111111_11111111_11111111_11111101

var locate_timer:Timer
signal controlled_avatar_changed(to: CharacterAvatar)
var controlled_avatar: CharacterAvatar :
	set(new_controlled_avatar):
		if new_controlled_avatar != controlled_avatar:
			if controlled_avatar:
				controlled_avatar.navigation_target_body.visible = false
			if new_controlled_avatar:
				camera_rig.target = new_controlled_avatar.body
				print("located character avatar ", controlled_avatar)
			
			controlled_avatar = new_controlled_avatar
			controlled_avatar_changed.emit(new_controlled_avatar)

# Overlay to draw between the game camera and UI elements. 
# Used to show black screen and images.
@export var overlay:Node :
	set(new_overlay):
		if new_overlay != overlay:
			if is_instance_valid(overlay):
				overlay.queue_free()
		
			if is_instance_valid(new_overlay):
				overlay_slot.add_child(new_overlay)
			
			overlay = new_overlay

@export var overlay_slot:CanvasLayer

func locate_character() -> void:
	if !is_inside_tree():
		return
	
	for c in get_tree().get_nodes_in_group(character_avatar_group):
		if c is CharacterAvatar:
			if c.character_data == character_data:
				controlled_avatar = c
				locate_timer.stop()
				return
	
	controlled_avatar = null
	locate_timer.start()

func _ready() -> void:
	add_to_group(group)
	
	locate_timer = Timer.new()
	locate_timer.one_shot = false
	locate_timer.wait_time = 1.0
	locate_timer.timeout.connect(locate_character)
	add_child(locate_timer)
	
	locate_character()

var camera_ray_out := false
var in_conversation := false
func _physics_process(_delta: float) -> void:
	var result:Dictionary
	if !in_conversation and controlled_avatar and Input.is_action_just_pressed("c_click_move"):
		var mouse_pos := camera_rig.camera.get_viewport().get_mouse_position()
		var ray_o := camera_rig.camera.project_ray_origin(mouse_pos)
		var ray_n := camera_rig.camera.project_ray_normal(mouse_pos)
		DebugDraw3d.draw_ray(ray_o, ray_o + ray_n * 100, Color.RED)
		var query := PhysicsRayQueryParameters3D.create(ray_o, ray_o + ray_n * 100, raycast_collision_flags)
		
		var space_state := camera_rig.camera.get_world_3d().direct_space_state
		result = space_state.intersect_ray(query)
		if result:
			DebugDraw3d.draw_box(result.position, Quaternion.IDENTITY, Vector3.ONE, Color.BLUE)
		
		camera_ray_out = true
	else:
		camera_ray_out = false
	
	if controlled_avatar and !result.is_empty():
		var position = result.position
		var collider = result.collider
		if collider is InteractableObject:
			position = collider.get_interact_position()
			controlled_avatar.navigate_to_interactable_rpc.rpc_id(1, collider.get_path())
		else:
			controlled_avatar.navigate_to_position_rpc.rpc_id(1, position)
		
		controlled_avatar.navigation_target_body.global_position = position
	
	if controlled_avatar:
		controlled_avatar.navigation_target_body.visible = !controlled_avatar.navigation_agent.is_target_reached()

func _on_avatar_request_dialogue(avatar:CharacterAvatar, other:InteractableObject):
	var source_cdata := Network.get_character_of()
	if source_cdata and avatar == controlled_avatar and other.conversation_path != "":
		in_conversation = true
		scrolling_dialogue.start_conversation(other.conversation_path)
		
