extends Node

## Give code completion suggestions for CodeEdit nodes that are
## 1. under `editor`
## 2. inside the group `hub_component_group`
## 3. inside one of the groups named in `identifier_regex.keys()`
## These nodes are scanned when they are added to the tree using the regex
## patterns in `identifier_regex` for their matching group(s), and then matches
## are added as completion suggestions for those groups.
##
## For example, if a CodeEdit node in the group "hub_completions_speaker" is
## added to the editor with the value "Adam", `identifier_pools["hub_completion_speaker"]`
## is set to `["Adam"]` and `identifier_counts["hub_completion_speaker]` is set
## to `[1]`.
##
## The counts are used to track when to remove completions that have 0 users and
## stop suggesting them.

@export var editor:GraphEdit
@export var editor_frame:Control
@export var hub_component_group := &"hub_component"

#Active code completion suggestions for different groups
var identifier_counts:Dictionary[StringName, PackedInt32Array]
var identifier_pools:Dictionary[StringName, PackedStringArray]
@export var identifier_regex:Dictionary[StringName, String]

var completion_menu:CustomGraphCompletionMenu

func validate_completion_target(target:Node) -> bool:
	if !is_instance_valid(target):
		return false
	
	if !(target is CodeEdit):
		return false
	
	if !target.is_in_group(hub_component_group):
		return false
	
	if !(editor.is_ancestor_of(target)):
		return false
	
	return true

func _ready() -> void:
	editor.child_entered_tree.connect(_on_candidate_pushed)
	editor.child_exiting_tree.connect(_on_candidate_popped)
	
	completion_menu = CustomGraphCompletionMenu.new()
	editor_frame.add_child(completion_menu)

func _on_candidate_pushed(c:Node):
	pass

func _on_candidate_popped(c:Node):
	pass

func _input(event: InputEvent) -> void:
	if event.is_action_pressed("ui_text_completion_query"):
		_on_completion_query()
		get_viewport().set_input_as_handled()

func _on_completion_query() -> void:
	var focused := get_viewport().gui_get_focus_owner()
	if !validate_completion_target(focused):
		return
	show_menu(focused as CodeEdit)

func show_menu(code_edit:CodeEdit) -> void:
	var word := CustomGraphCompletionMenu.get_word(code_edit)
	var candidates := gather_candidates(word)
	print("completing \"", word, "\" -> ", candidates)
	if candidates.is_empty():
		completion_menu.close()
		return
	completion_menu.open(code_edit, candidates)

#Canned candidates for now. Later this reads the pools of the groups
#that the CodeEdit belongs to.
func gather_candidates(word:String) -> Array[String]:
	var pool:Array[String] = ["Adam", "Adams", "Beatrice"]
	return pool.filter(func(candidate:String) -> bool: return candidate.to_lower().begins_with(word.to_lower()))
