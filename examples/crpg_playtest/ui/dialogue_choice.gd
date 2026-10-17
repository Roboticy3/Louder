class_name DialogueChoice extends Button

signal chosen(row: int)

var _row := 0

func setup(row: int, text: String) -> void:
	_row = row
	self.text = text

func _ready() -> void:
	pressed.connect(_on_pressed)

func _on_pressed() -> void:
	chosen.emit(_row)