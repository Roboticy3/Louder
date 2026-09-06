class_name ScrollingDialogueFeed extends Node

const BodyProperties := PSObjects.BodyProperties

@export var insert_index := -2
@export var remove_index := 1
@export var dialogue_item:PackedScene
@export var max_items := 100
@export var scroll_container:ScrollContainer
@export var debug_display:Node
@export var spectator:=false

var item_count := 0
var top:Node = null

func push_item(body:Dictionary[BodyProperties, Variant]):
	if debug_display and debug_display.has_method("set_body"):
		debug_display.set_body(body)
	
	if top and top.has_method("clear_choices"):
		top.clear_choices.call_deferred()
	
	while item_count >= max_items:
		var child := get_child(remove_index)
		remove_child(child)
		child.queue_free()
		item_count -= 1
	
	var item := dialogue_item.instantiate()
	if spectator:
		item.set("disabled", true)
	if item.has_method("import"):
		item.import(body)
	if item.has_signal("choice_pressed"):
		item.choice_pressed.connect(_on_item_choice_pressed)
	item.name = str(body.get(BodyProperties.position, -1))
	item_count += 1
	print("MainDialogueView: displaying position ", item.name)
	add_child(item)
	move_child(item, insert_index)
	top = item
	
	if scroll_container:
		var tween := create_tween()
		var vscrollbar := scroll_container.get_v_scroll_bar()
		tween.tween_property(vscrollbar, "value", vscrollbar.max_value, 0.5).set_ease(Tween.EASE_IN_OUT)
		#fun fact, if the scroll container's vertical scroll mode is not set to "Reserve", the engine
		#slowly and painfully crashes: https://github.com/godotengine/godot/issues/108407
		#
		#two days of my life!
		

func _on_item_choice_pressed(choice: String) -> void:
	print(multiplayer.get_unique_id(), ": pressed choice ", choice, "!")
	var controller := ControllerEventLayer.get_connected_controller()
	if is_instance_valid(controller):
		ControllerEventLayer.request_choice(controller, choice)

func clear():
	for c in get_children():
		remove_child(c)
		c.queue_free()
	item_count = 0
