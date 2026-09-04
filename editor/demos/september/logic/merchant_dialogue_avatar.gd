extends Node

@export var config:PSConfig
@export var controller:PSController
@export var group := &"merchant_dialogue_avatar"

func _ready():
	add_to_group(group)
