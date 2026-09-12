extends Node

@export var file_dialog_button:BaseButton
@export var file_dialog:FileDialog
@export var line_edit:LineEdit

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	file_dialog_button.pressed.connect(file_dialog.show)
	file_dialog.file_selected.connect(_on_file_selected)

func _on_file_selected(file_path:String):
	line_edit.text = file_path
	line_edit.text_submitted.emit(file_path)
