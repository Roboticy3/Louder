class_name CharacterAvatar extends Node3D

@export var character_data:CharacterData
@export var body:CharacterBody3D
@export var navigation_agent:NavigationAgent3D
@export var walk_speed := 3.0

@export var group := &"character_avatar"

func _ready() -> void:
	add_to_group(group)

var navigation_input := Vector3.ZERO
func _physics_process(delta: float) -> void:
	var next_position := navigation_agent.get_next_path_position()
	var current_position := body.global_position
	var error := next_position - current_position
	navigation_input = error.normalized() * walk_speed
	
	if error.length() > 0.1:
		body.velocity = navigation_input
	else:
		body.velocity = Vector3.ZERO
	
	body.velocity += Vector3.DOWN * delta
	
	body.move_and_slide()
	
	DebugDraw3d.draw_box(navigation_agent.target_position, Quaternion.IDENTITY, Vector3.ONE, Color.BLACK)
	DebugDraw3d.draw_box(next_position, Quaternion.IDENTITY, Vector3.ONE, Color.RED)
	
	
