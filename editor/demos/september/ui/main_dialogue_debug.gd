extends Control

const BodyProperties := PSObjects.BodyProperties
const FILTER:Dictionary[BodyProperties, bool] = {
	BodyProperties.branch: true,
	BodyProperties.position: true,
	BodyProperties.success: true,
}

@export var data_text:RichTextLabel
@export var speaker_text:RichTextLabel

func _ready() -> void:
	ControllerEventLayer.event.connect(_on_controller_event)
	_on_controller_event(0, ControllerEventLayer.get_connected_controller())

func _on_controller_event(_type, controller:PSController) -> void:
	
	var current_controller := ControllerEventLayer.get_connected_controller()
	if !current_controller or current_controller != controller:
		return
	
	var data := controller.data
	
	var new_data_text:=""
	for k in FILTER:
		if !(k in data): continue
		var key_name = BodyProperties.find_key(k)
		new_data_text += "[b]%s[/b]: %s\n" % [str(key_name), str(data[k])]
	data_text.text = new_data_text
	
	var speak_as := controller.speak_as
	var listen_to := controller.listen_to
	var new_speaker_text:=""
	for i in speak_as.size():
		new_speaker_text += "[b]%s %d[/b]: %s\n" % ["speak_as", i, speak_as[i]]
	for i in listen_to.size():
		new_speaker_text += "[b]%s %d[/b]: %s\n" % ["listen_to", i, listen_to[i]]
	speaker_text.text = new_speaker_text
	
