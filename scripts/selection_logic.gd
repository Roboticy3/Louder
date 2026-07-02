extends Node

@export var editor:GraphEdit

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	editor.set_selected(null)
	editor.get_viewport().gui_focus_changed.connect(_on_gui_focus_changed)
	editor.node_selected.connect(_on_node_selected)
	
	await get_tree().process_frame
	for n in editor.get_children():
		if n is GraphElement and n.selectable:
			editor.set_selected(n)
			break

func _on_node_selected(node:GraphElement):
	if !node:
		return
	var focus := editor.get_viewport().gui_get_focus_owner()
	if !focus or !node.is_ancestor_of(focus):
		if node.get_child_count() == 0:
			node.selected = false
		else:
			for c in node.get_children():
				if c is Control and c.focus_mode & 2:
					c.grab_focus.call_deferred()
					break

func _on_gui_focus_changed(to:Control):
	if !editor.is_ancestor_of(to):
		return
	
	#first element here is a GraphNode by convention
	var relative_path := editor.get_path_to(to)
	var graph_node_path := relative_path.slice(0, 1)
	var graph_node = editor.get_node_or_null(graph_node_path)
	
	if !(graph_node is GraphNode):
		editor.set_selected(null)
		return
	
	if !graph_node.selected:
		editor.set_selected(graph_node)
