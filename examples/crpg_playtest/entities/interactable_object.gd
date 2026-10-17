class_name InteractableObject extends PhysicsBody3D

@export var interact_position_marker:Marker3D

func get_interact_position():
	return interact_position_marker.global_position
