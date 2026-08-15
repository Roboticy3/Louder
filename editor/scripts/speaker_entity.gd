class_name SpeakerEntity extends NetworkEntity

@export var private_check:CheckBox
@export var color_picker:ColorPickerButton

const TYPE := "speaker"

func get_entity_type() -> String:
	return TYPE

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	super._ready()
	
	
