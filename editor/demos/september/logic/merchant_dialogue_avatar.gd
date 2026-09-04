extends Node

@export var config:PSConfig

func get_faux_authority() -> int:
	return int(name)
