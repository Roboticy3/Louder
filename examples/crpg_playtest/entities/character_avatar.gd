class_name CharacterAvatar extends Node3D

@export var character_data:CharacterData
@export var body:CharacterBody3D
@export var navigation_agent:NavigationAgent3D
@export var animation_tree:CharacterAvatarAnimationTree

@export var walk_speed := 3.0

@export var group := &"character_avatar"
@export var character_controller_group := &"character_controller"

func _ready() -> void:
	add_to_group(group)
	navigation_agent.path_changed.connect(_on_path_changed)
	navigation_agent.target_reached.connect(_on_target_reached)

var navigation_input := Vector3.ZERO

enum NavigationState {
	Idle,
	Navigating,
	Turning,
}
var nav_state := NavigationState.Idle

func _physics_process(delta: float) -> void:
	var gravity := Vector3.DOWN
	
	match nav_state:
		NavigationState.Navigating:
			var next_position := navigation_agent.get_next_path_position()
			var current_position := body.global_position
			var error := next_position - current_position
			navigation_input = error.normalized() * walk_speed
			
			if error.length() > 0.1:
				body.velocity = navigation_input
				
				body.look_at(body.global_position + body.velocity, Vector3.UP, true)
				DebugDraw3d.draw_arrow(body.global_position, -body.global_basis.z, Color.BLUE)
			else:
				body.velocity = Vector3.ZERO
			
			DebugDraw3d.draw_box(navigation_agent.target_position, Quaternion.IDENTITY, Vector3.ONE, Color.BLACK)
			DebugDraw3d.draw_box(next_position, Quaternion.IDENTITY, Vector3.ONE, Color.RED)
		NavigationState.Idle: 
			navigation_input = Vector3.ZERO
			body.velocity = Vector3.ZERO
	body.move_and_slide()

func _process(_delta: float) -> void:
	match nav_state:
		NavigationState.Idle:
			animation_tree.idling = true
		NavigationState.Navigating:
			animation_tree.idling = false
			animation_tree.walk_speed = body.velocity.length()

var avoid_rng := RandomNumberGenerator.new()
func avoid_occupied(pos:Vector3, offset:Vector3, seed:int) -> Vector3:
	avoid_rng.seed = seed
	var space_state = get_world_3d().direct_space_state
	var occupation_query := PhysicsPointQueryParameters3D.new()
	occupation_query.collision_mask = body.collision_mask
	occupation_query.position = pos + offset
	
	var occupation_result := space_state.intersect_point(occupation_query)
	while !occupation_result.is_empty():
		pos += Vector3.FORWARD.rotated(Vector3.UP, avoid_rng.randf_range(0.0, 2 * PI)) * 1.5
		occupation_query.position = pos + offset
		occupation_result = space_state.intersect_point(occupation_query)
		occupation_result = space_state.intersect_point(occupation_query)
	
	return pos


@rpc("call_local", "any_peer", "reliable")
func navigate_to_position_rpc(to:Vector3):
	if !multiplayer.is_server():
		return
	var remote_id := multiplayer.get_remote_sender_id()
	var remote_data := Network.get_character_of(remote_id)
	if remote_data == character_data:
		_navigate_to_position.rpc(to)

@rpc("call_local", "authority", "reliable")
func _navigate_to_position(to:Vector3):
	to = avoid_occupied(to, body.global_basis.y, int(to.x + to.y + to.z))
	intent_to_interact = null
	navigation_agent.target_position = to
	nav_state = NavigationState.Navigating

@rpc("call_local", "any_peer", "reliable")
func navigate_to_interactable_rpc(interactable_path:NodePath):
	if !multiplayer.is_server():
		return

	var remote_id := multiplayer.get_remote_sender_id()
	var remote_data := Network.get_character_of(remote_id)
	if remote_data == character_data:
		_navigate_to_interactable.rpc(interactable_path)

var intent_to_interact:InteractableObject
@rpc("call_local", "authority", "reliable")
func _navigate_to_interactable(interactable_path:NodePath):
	var i = get_node_or_null(interactable_path)
	if !(i is InteractableObject):
		intent_to_interact = null
		return
	
	_navigate_to_position(i.get_interact_position())
	intent_to_interact = i

func _on_path_changed():
	print("path changed")

func _on_target_reached():
	if intent_to_interact:
		nav_state = NavigationState.Turning
		
		body.look_at.call_deferred(intent_to_interact.global_position, Vector3.UP, true)
		DebugDraw3d.draw_arrow_line(intent_to_interact.get_interact_position(), intent_to_interact.global_position, Color.RED)
		DebugDraw3d.draw_arrow(body.global_position, -body.global_basis.z * 3, Color.BLUE)
		
		#Start dialogue. Connected controller should see this and trigger dialogue UI
		print("starting dialogue with ", intent_to_interact)
		get_tree().call_group(character_controller_group, &"_on_avatar_request_dialogue", self, intent_to_interact)
	
	intent_to_interact = null
	nav_state = NavigationState.Idle
