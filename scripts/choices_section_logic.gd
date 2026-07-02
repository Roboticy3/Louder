class_name ChoicesSectionLogic extends Node
@export var hub:GraphNode
@export var spin_box:SpinBox
@export var start_check:CheckBox
@export var child_index := 4
@export var spoke_child_index := 2

var choices:Array[LineEdit] = []
signal choices_changed()

func add_choice_auto():
	var string := "(Choice " + str(choices.size()) + ")"
	add_choice(string)

func add_choice(text:String):
	var le := LineEdit.new()
	le.text = text
	hub.add_child(le)
	hub.move_child(le, child_index + choices.size())
	hub.set_slot(
		child_index + choices.size(), 
		false,
		0,
		Color.WHITE, 
		true, 
		1, 
		Color.WHITE)
	choices.append(le)
	choices_changed.emit()

func remove_choice(index:int):
	if index > choices.size():
		return
	hub.set_slot(
		child_index + index, 
		false, 
		0, 
		Color.WHITE,
		false,
		0,
		Color.WHITE)
	hub.remove_child.call_deferred(choices[index])
	choices_changed.emit()
	
	choices.remove_at(index)

func update_default_choices():
	if choices.is_empty():
		hub.set_slot(
			spoke_child_index,
			!start_check.button_pressed, 
			0 if start_check.button_pressed else 1, 
			Color.WHITE,
			true,
			1,
			Color.WHITE
		)

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

func _ready() -> void:
	spin_box.value_changed.connect(set_choice_count)
	start_check.pressed.connect(update_default_choices)
