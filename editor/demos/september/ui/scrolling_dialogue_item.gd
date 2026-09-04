extends Node

const BodyProperties := PSObjects.BodyProperties

@export var element_container:Container

#replace children in the element container
func import(data:Dictionary[BodyProperties, Variant]):
	for c in get_children():
		remove_child(c)
		c.queue_free()
