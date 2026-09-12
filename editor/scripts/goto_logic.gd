extends Node

@export var go_to_id:LineEdit
@export var editor:GraphEdit

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	go_to_id.text_changed.connect(_on_text_changed)

func _on_text_changed(new_text:String):
	var id := int(new_text)
	var node:Node
	while id < IdSystem.next_id:
		node = IdSystem.get_assigned(id)
		if node is NetworkEntity:
			break
		id += 1
	
	if node is NetworkEntity:
		goto(node)

func goto(graph_node:NetworkEntity) -> void:
	var center:Vector2 = graph_node.position_offset + graph_node.size * 0.5
	editor.scroll_offset = center * editor.zoom - editor.size * 0.5
