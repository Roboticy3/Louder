class_name PSConversationState extends Resource

var history:PackedInt32Array
@export var covered:Dictionary[int, bool]
@export var variables:Dictionary

func _on_traverse(to_position:int) -> bool:
	var exists := to_position in covered
	if !exists: covered[to_position] = true
	history.push_back(to_position)
	return exists
