extends Control

var slot:Node = null

@export var character_ui_fallback:PackedScene

@export var character_ui_path_format := "res://character_ui/%s_ui.tres"
@export var lower_ui_path_format := true

@export var preload_ui_paths:Array[CharacterData]
var loaded_ui:Dictionary[StringName, Node]

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	for p in preload_ui_paths:
		var ui := ui_from_character_name(p.name)
		if !ui:
			continue
		loaded_ui[p.name] = ui
	
	Network.selection_accepted.connect(_on_selection_accepted)

func _on_selection_accepted(character_name:StringName) -> void:
	if character_name in loaded_ui:
		if is_instance_valid(loaded_ui[character_name]):
			set_slot(loaded_ui[character_name])
	
	set_slot(ui_from_character_name(character_name))

func ui_from_character_name(character_name:StringName) -> Node:
	if lower_ui_path_format:
		character_name = character_name.to_lower()
	
	var path := character_ui_path_format % [character_name]
	var packed = load(path)
	if !(packed is PackedScene):
		push_error("Could not find dynamic character ui from path ", path, " for character ", character_name)
		return character_ui_fallback.instantiate()
	
	if !packed.can_instantiate():
		push_error("Could not instantiate dynamic character ui from path ", path, " for character ", character_name)
		return character_ui_fallback.instantiate()
	
	return packed.instantiate()

func set_slot(to:Node):
	if is_instance_valid(slot):
		slot.queue_free()
	slot = to
	add_child(slot)
