class_name CharacterAvatarControllerSingle extends Node

@export var character_avatar_group := &"character_avatar"
@export var character_data:CharacterData

@export var camera_rig:CameraRigSimple 

var locate_timer:Timer
signal controlled_avatar_changed(to: CharacterAvatar)
var controlled_avatar: CharacterAvatar :
	set(new_controlled_avatar):
		if new_controlled_avatar != controlled_avatar:
			controlled_avatar = new_controlled_avatar
			
			if controlled_avatar:
				camera_rig.target = controlled_avatar.body
				print("located character avatar ", controlled_avatar)
			
			
			controlled_avatar_changed.emit(new_controlled_avatar)

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
	locate_timer = Timer.new()
	locate_timer.one_shot = false
	locate_timer.wait_time = 1.0
	locate_timer.timeout.connect(locate_character)
	add_child(locate_timer)
	
	locate_character()

var camera_ray_out := false
func _physics_process(_delta: float) -> void:
	var result:Dictionary
	if Input.is_action_pressed("c_click_move"):
		var mouse_pos := camera_rig.camera.get_viewport().get_mouse_position()
		var ray_o := camera_rig.camera.project_ray_origin(mouse_pos)
		var ray_n := camera_rig.camera.project_ray_normal(mouse_pos)
		DebugDraw3d.draw_ray(ray_o, ray_o + ray_n * 100, Color.RED)
		var query := PhysicsRayQueryParameters3D.create(ray_o, ray_o + ray_n * 100)
		
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
			controlled_avatar.navigate_to_interactable(collider)
		else:
			controlled_avatar.navigate_to_position(position)
		
