extends Node

const BodyProperties := PSObjects.BodyProperties

@export var insert_index := 1
@export var dialogue_item:PackedScene
@export var max_items := 100

var item_count := 0
var last_item = null

func push_item(body:Dictionary[BodyProperties, Variant]):
	if last_item and last_item.has_method("clear_choices"):
		last_item.clear_choices()
	
	while item_count >= max_items:
		var child := get_child(insert_index)
		remove_child(child)
		child.queue_free()
		item_count -= 1
	
	dialogue_item.instantiate()
