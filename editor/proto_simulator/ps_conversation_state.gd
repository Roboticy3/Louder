class_name PSConversationState extends Resource

var history:PackedInt32Array
@export var covered:Dictionary[int, bool]

@export var variables:Dictionary[StringName, Variant]
var scope:PSVariableScope = PSVariableScope.new(variables)

func _on_traverse(to_position:int) -> bool:
	var exists := to_position in covered
	if !exists: covered[to_position] = true
	history.push_back(to_position)
	print("moved!")
	print(self)
	return exists

func _to_string() -> String:
	return "PSConversationState(\n history: %s\n)" % [str(history)]
