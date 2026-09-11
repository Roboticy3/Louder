extends LineEdit

func _ready() -> void:
	text_submitted.connect(_on_text_submitted)

func _on_text_submitted(new_text: String) -> void:
	var controller := ControllerEventLayer.get_connected_controller()
	if !is_instance_valid(controller):
		return
	var position := int(new_text)
	ControllerEventLayer.request_warp(controller, position)
	clear()
