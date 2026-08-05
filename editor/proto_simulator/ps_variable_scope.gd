class_name PSVariableScope extends Object

var variables:Dictionary[StringName, Variant]
const DEFAULT_VALUE := 0

func _init(variables_source:Dictionary[StringName, Variant]):
	variables = variables_source

func _get(property: StringName) -> Variant:
	if property in variables:
		return variables[property]
	else:
		variables[property] = DEFAULT_VALUE
		return DEFAULT_VALUE

func _set(property: StringName, value: Variant) -> bool:
	variables[property] = value
	return true

func _to_string():
	return str(variables)
