class_name ScrollingDialogueFeed extends Node

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
	
	var item := dialogue_item.instantiate()
	if item.has_method("import"):
		item.import(body)
	add_child(item)
	move_child(item, insert_index + item_count)

func clear():
	for c in get_children():
		remove_child(c)
		c.queue_free()
	item_count = 0
