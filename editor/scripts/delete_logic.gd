extends Node

@export var editor:GraphEdit

func _input(event: InputEvent) -> void:
	if event.is_action_pressed("ui_graph_delete"):
		var last_deleted = null
		for c in editor.get_children():
			if c is GraphElement and c.selected:
				c.queue_free()
				last_deleted = c
		
		#if the last deleted node exists and has connections, go to those connections
		var auto_next_found := false
		if last_deleted is Node:
			var connections := editor.get_connection_list_from_node(last_deleted.name)
			for c in connections:
				
				var to_node:GraphNode = editor.get_node(NodePath(c["to_node"]))
				var from_node:GraphNode = editor.get_node(NodePath(c["from_node"]))
				
				if !to_node.selected && from_node == last_deleted && to_node != last_deleted:
					editor.set_selected(to_node)
					auto_next_found = true
				elif !from_node.selected && to_node == last_deleted && from_node != last_deleted:
					editor.set_selected(from_node)
					auto_next_found = true
				
				if auto_next_found:
					break
		
		#otherwise, try to find a start node.
		if !auto_next_found:
			for n in editor.get_children():
				if n is CustomGraphHub:
					var start_check:CheckBox = n.get_start_check()
					if start_check and start_check.button_pressed:
						editor.set_selected(n)
						auto_next_found = true
						break
		
		#otherwise otherwise, just select anything
		if !auto_next_found:
			for n in editor.get_children():
				if n is NetworkEntity:
					editor.set_selected(n)
					break
