class_name ScrollingDialogueChoice extends VBoxContainer

@export var height:Control
@export var label:RichTextLabel
@export var index: Label
@export var button: BaseButton
@export var background: NinePatchRect

@export var default_color := Color.WHITE
@export var mouse_color := Color.BROWN


signal choice_pressed(choice:String)

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	height.resized.connect(_on_height_resized, CONNECT_DEFERRED)
	button.pressed.connect(_on_pressed)
	button.mouse_entered.connect(_on_entered)
	button.mouse_exited.connect(_on_exited)
	button.focus_entered.connect(_on_entered)
	button.focus_exited.connect(_on_exited)

	_on_exited()

func _on_height_resized():
	if height.size.y != custom_minimum_size.y:
		custom_minimum_size.y = height.size.y

func _on_pressed():
	#print("choice ", choice, " pressed!")
	choice_pressed.emit(choice)

func _on_entered():
	label.modulate = mouse_color
	index.modulate = mouse_color

func _on_exited():
	if has_focus():
		return
	label.modulate = default_color
	index.modulate = default_color

var choice:String
func set_choice(to:String):
	choice = to
	print(self, ": set choice to ", choice)
	
	if to.is_empty():
		label.text = "CONTINUE"
	else:
		label.text = to

var action := &""
func set_index(to:int) -> bool:
	if to < 1 or to > 10:
		index.text = ""
		action = &""
		return false
	
	action = StringName("dialogue_%d" % to)
	index.text = "%d." % to
	
	return true

func _input(event:InputEvent):
	if !action.is_empty():
		if event.is_action_pressed(action):
			button.grab_focus.call_deferred()
		elif button.has_focus() and event.is_action_released(action):
			print(self, ": forcing choice ", choice)
			button.pressed.emit()

func set_texture(to:Texture):
	if background: background.texture = to
