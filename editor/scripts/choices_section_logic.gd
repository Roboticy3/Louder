class_name ChoicesSectionLogic extends Node
@export var hub:GraphNode
@export var spin_box:SpinBox
@export var start_check:CheckBox
@export var start_check_default:bool = false
@export var child_index := 4
@export var spoke_child_index := 2

@export var port_change_listener_group := &"port_change_listener"
@export var port_change_listener_method := &"_on_port_changed"

var choices:Array[Label] = []
signal choices_changed()

func is_start_checked():
	if is_instance_valid(start_check):
		return start_check.get("button_pressed")
	return start_check_default

func add_choice_auto():
	var placeholder_text := "(Choice " + str(choices.size()) + ")"
	add_choice("", placeholder_text)

func add_choice(text:String, placeholder_text:String = "(Choice)"):
	var le := Label.new()
	le.set("text", text)
	le.set("placeholder_text", placeholder_text)
	le.set("inliner", true)
	hub.add_child(le)
	hub.move_child(le, child_index + choices.size())
	hub.set_slot(
		child_index + choices.size(), 
		false,
		0,
		Color.WHITE, 
		true, 
		1, 
		Color.WHITE
	)
	if is_inside_tree():
		get_tree().call_group.call_deferred(port_change_listener_group, &"_on_port_changed", hub.name, choices.size())
	choices.append(le)
	choices_changed.emit()

func remove_choice(index:int):
	if index > choices.size():
		return
	
	var slot_idx := child_index + index
	
	hub.set_slot(
		slot_idx, 
		false, 
		0, 
		Color.WHITE,
		false,
		0,
		Color.WHITE
	)
	if is_inside_tree():
		get_tree().call_group.call_deferred(port_change_listener_group, &"_on_port_changed", hub.name, index)

	hub.remove_child.call_deferred(choices[index])
	choices_changed.emit()
	choices.remove_at(index)

func update_default_choices():
	hub.set_slot.call_deferred(
		spoke_child_index,
		true,
		1,
		Color.WHITE,
		choices.is_empty(),
		0 if !choices.is_empty() else 1,
		Color.WHITE
	)
	hub.reset_size.call_deferred()
	if is_inside_tree():
		get_tree().call_group.call_deferred(port_change_listener_group, &"_on_port_changed", hub.name, 0)

func set_choice_count(to:float):
	var desired := int(to)
	var current := choices.size()
	
	if desired < current:
		for i in range(current - 1, desired - 1, -1):
			remove_choice(i)
	elif desired > current:
		for i in range(current, desired):
			add_choice_auto()
	
	update_default_choices()
	
	if is_inside_tree():
		await get_tree().process_frame
		hub.reset_size()

func get_choice_from_label(label:Label) -> String:
	if !is_instance_valid(label):
		return ""
	return label.text

func choice_to_port_out(choice:String) -> int:
	if choice.is_empty() and choices.is_empty():
		return 0 #"default" or "continue"
	
	for i in choices.size():
		var label := choices[i]
		if get_choice_from_label(label) == choice:
			return i
	return -1

func port_out_to_choice(port:int) -> String:
	if port >= 0 && port < choices.size():
		return get_choice_from_label(choices[port])
	return ""

func port_in() -> int:
	return 0

var is_end := false
func set_end(_new_is_end:bool) -> void:
	is_end = _new_is_end
	if is_end:
		spin_box.value = 0
	hub.set_slot_enabled_right.call_deferred(spoke_child_index, !is_end)

func _ready() -> void:
	if is_instance_valid(spin_box):
		spin_box.value = choices.size()
		spin_box.value_changed.connect(set_choice_count)
	if is_instance_valid(start_check):
		start_check.pressed.connect(update_default_choices)
	
	update_default_choices.call_deferred()
