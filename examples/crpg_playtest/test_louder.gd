extends Node


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	var ctx := LouderContext.new()
	var fa := FileAccess.open("res://conversations/jail.json", FileAccess.READ)
	var jail := ctx.make_name(&"jail")
	var start := ctx.make_name(&"start")
	var conversation := ctx.load_conversation(jail, fa.get_as_text())

	print("name: ", jail)
	print("conversation: ", conversation)
	print("start: ", start)
	
	var hub := ctx.next_hub_from_tags(conversation, [start], false, -1)
	while hub != -1:
		print("start hub: ", hub)
		hub = ctx.next_hub_from_tags(conversation, [start], false, hub)
