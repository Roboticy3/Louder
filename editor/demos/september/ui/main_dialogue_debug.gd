extends Control

const BodyProperties := PSObjects.BodyProperties
const FILTER:Dictionary[BodyProperties, bool] = {
	BodyProperties.branch: true,
	BodyProperties.position: true,
	BodyProperties.success: true,
}

@export var text:RichTextLabel

func set_body(body:Dictionary[BodyProperties, Variant]) -> void:
	var new_text:=""
	for k in body:
		if !(k in FILTER): continue
		var key_name = BodyProperties.find_key(k)
		new_text += "[b]%s[/b]: %s\n" % [str(key_name), str(body[k])]
	text.text = new_text
