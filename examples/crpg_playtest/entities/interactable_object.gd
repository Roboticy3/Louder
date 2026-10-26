class_name InteractableObject extends PhysicsBody3D

@export var interact_position_marker:Marker3D
@export_file("*.json") var conversation_path:String

# Server-side bookkeeping: which conversation this interactable is hosting, so
# that a second avatar reaching it joins the existing one instead of starting
# another. Only the host reads or writes this.
var conversation_id: int = -1

func get_interact_position() -> Vector3:
	return interact_position_marker.global_position
