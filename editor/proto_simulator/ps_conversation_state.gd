class_name PSConversationState extends Resource

@export var branches:Dictionary[String, PSConversationBranch]
@export var covered:Dictionary[int, bool]

@export var variables:Dictionary[StringName, Variant]
var scope:PSVariableScope = PSVariableScope.new(variables)

func _init() -> void:
	_reset_branches()

func _reset_branches():
	branches = {
		"": PSConversationBranch.new()
	}

func clear():
	variables.clear()
	covered.clear()
	_reset_branches()

func _on_traverse(to_position:int) -> bool:
	var exists := to_position in covered
	if !exists: covered[to_position] = true
	print("moved!")
	print(self)
	return exists

func _to_string() -> String:
	return "PSConversationState(\n variables: %s\n)" % [str(variables)]
