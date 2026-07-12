extends Node

var next_id := 0
var mapping:Dictionary[int, Node]

func assign_id(node:Node, id:=next_id, old_id:=-1) -> int:
	if mapping.get(old_id) == node:
		mapping.erase(old_id)
	mapping[id] = node
	next_id = maxi(next_id, id + 1)
	return id

func get_assigned(id:int) -> Node:
	return mapping.get(id)

func clear_ids() -> void:
	mapping.clear()
	next_id = 0
