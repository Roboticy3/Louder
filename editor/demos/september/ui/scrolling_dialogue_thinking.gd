class_name ScrollingDialogueThinking extends Control

@export var thinking_label:Label

func update_thinkers(thinkers:Array) -> void:
	var thinking_text := ""
	if thinkers.is_empty():
		pass
	elif thinkers.size() > 1: 
		thinking_text = "%s" % thinkers[0]
		for i in range(1, thinkers.size() - 1):
			thinking_text += ", %s" % thinkers[i]
		thinking_text += " and %s are thinking..." % thinkers[-1]
	else:
		thinking_text = "%s is thinking..." % thinkers[0]
	thinking_label.text = thinking_text
