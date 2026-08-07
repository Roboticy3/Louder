extends FoldableContainer


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	
	folded = true
	var nodes := find_children("", "CustomGraphNodeTextEdit", true)
	for n in nodes:
		if n.get("text") != "":
			folded = false
			break
