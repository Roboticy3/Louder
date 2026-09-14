extends Node

@export var options:OptionButton
@export var line_edit:LineEdit
@export var mapping:Dictionary[String, String]

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	options.add_item("")
	for m in mapping:
		options.add_item(m)
	options.text = "Defaults"
	options.item_selected.connect(_on_item_selected)

func _on_item_selected(index:int) -> void:
	var key := options.get_item_text(index)
	if key in mapping:
		key = mapping[key]
	line_edit.text = key
	line_edit.text_submitted.emit(line_edit.text)
