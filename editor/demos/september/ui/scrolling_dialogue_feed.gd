class_name ScrollingDialogueFeed extends Node

const BodyProperties := PSObjects.BodyProperties

@export var insert_index := -2
@export var remove_index := 1
@export var dialogue_item:PackedScene
@export var max_items := 100
@export var scroll_container:ScrollContainer
@export var debug_display:Node

var item_count := 0
var top:Node = null

func push_item(body:Dictionary[BodyProperties, Variant]):
	if debug_display and debug_display.has_method("set_body"):
		debug_display.set_body(body)
	
	if top and top.has_method("clear_choices"):
		top.clear_choices()
	
	while item_count >= max_items:
		var child := get_child(remove_index)
		remove_child(child)
		child.queue_free()
		item_count -= 1
	
	var item := dialogue_item.instantiate()
	if item.has_method("import"):
		item.import(body)
	if item.has_signal("choice_pressed"):
		item.choice_pressed.connect(_on_item_choice_pressed)
	item.name = str(body.get(BodyProperties.position, -1))
	print("MainDialogueView: displaying position ", item.name)
	add_child(item)
	move_child(item, insert_index)
	top = item
	
	var tween := create_tween()
	var vscrollbar := scroll_container.get_v_scroll_bar()
	tween.tween_property(vscrollbar, "value", vscrollbar.max_value, 0.5).set_ease(Tween.EASE_IN_OUT)
	

func _on_item_choice_pressed(choice: String) -> void:
	var controller := ControllerEventLayer.get_connected_controller()
	if is_instance_valid(controller):
		ControllerEventLayer.request_choice(controller, choice)

func clear():
	for c in get_children():
		remove_child(c)
		c.queue_free()
	item_count = 0
