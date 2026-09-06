class_name PSDialogueState extends Resource

@export var covered:Dictionary[int, bool]

@export var variables:Dictionary[StringName, Variant]
var scope:PSVariableScope = PSVariableScope.new(variables)

func clear():
	variables.clear()
	covered.clear()

func _to_string() -> String:
	return "PSDialogueState(\n variables: %s\n)" % [str(variables)]
