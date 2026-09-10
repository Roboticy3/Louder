extends Node

const SEARCH_GROUP := &"hub_text_edit"

@export var editor:GraphEdit
@export var search_bar:CanvasItem
@export var search_bar_line_edit:LineEdit

var matches:Array[CustomGraphNodeTextEdit] = []
var current_match := -1
var query := ""

func _ready() -> void:
	search_bar_line_edit.text_changed.connect(_on_query_changed)
	search_bar_line_edit.focus_exited.connect(_on_line_edit_focus_lost)

func _input(event: InputEvent) -> void:
	if event.is_action_pressed("graph_search_forward"):
		_search(true)
	elif event.is_action_pressed("graph_search_backward"):
		_search(false)

func _search(forward: bool) -> void:
	if !search_bar.visible:
		show_search_bar()
	if forward:
		search_bar_line_edit.select_all()
		next_match()
	else:
		previous_match()

func show_search_bar() -> void:
	search_bar.visible = true
	search_bar_line_edit.grab_focus()
	search_bar_line_edit.select_all()
	catalog_matches()

func _on_query_changed(_new_text: String) -> void:
	catalog_matches()

func _on_line_edit_focus_lost() -> void:
	search_bar.visible = false
	matches.clear()

func catalog_matches() -> void:
	matches.clear()
	current_match = -1
	query = search_bar_line_edit.text
	if query.is_empty():
		return
	for candidate in get_tree().get_nodes_in_group(SEARCH_GROUP):
		var valid_type := candidate is CustomGraphNodeTextEdit
		var ancestor := editor.is_ancestor_of(candidate)
		var search := false
		if valid_type:
			search = (candidate.search(query, 0, 0, true) != -Vector2i.ONE)
		if valid_type and ancestor and search:
			matches.append(candidate)

func next_match() -> void:
	if matches.is_empty():
		return
	if current_match < 0:
		current_match = 0
	else:
		current_match = (current_match + 1) % matches.size()
	focus_match()

func previous_match() -> void:
	if matches.is_empty():
		return
	if current_match < 0:
		current_match = matches.size() - 1
	else:
		current_match = posmod(current_match - 1, matches.size())
	focus_match()

func focus_match() -> void:
	var text_edit := matches[current_match] as CustomGraphNodeTextEdit
	var pos := text_edit.search(query, 0, 0, true)
	if pos.x >= 0:
		text_edit.select(pos.y, pos.x, pos.y, pos.x + query.length())
	var graph_node := text_edit.get_parent()
	while graph_node != null and not (graph_node is GraphNode):
		graph_node = graph_node.get_parent()
	if graph_node is GraphNode:
		var center:Vector2 = graph_node.position_offset + graph_node.size * 0.5
		editor.scroll_offset = center * editor.zoom - editor.size * 0.5
