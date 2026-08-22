extends RichTextLabel

@export var simulator: PSSimulator

@export var show_conversation := true
@export var show_position := true
@export var show_choices := true
@export var show_variables := true

func _ready() -> void:
	if not simulator:
		return
	if not simulator.is_node_ready():
		await simulator.ready
	add_to_group(&"ps_controller")
	update()

func update() -> void:
	await get_tree().process_frame

	if not simulator:
		text = "No simulator assigned."
		return
	var parts: PackedStringArray = []
	if show_conversation:
		parts.append("[b]Conversation:[/b] %s" % simulator.conversation_path)
	if show_position:
		parts.append("[b]Position:[/b] %s" % str(simulator.get_position()))
	if show_choices:
		var raw_choices := simulator.list_choices(null, simulator.position, false)
		var choice_texts: PackedStringArray = []
		for c in raw_choices:
			choice_texts.append(str(c.get("from_choice", "?")))
		parts.append("[b]Choices:[/b]\n%s" % "\n".join(choice_texts) if choice_texts.size() else "[b]Choices:[/b] (none)")
	if show_variables and simulator.state:
		var vars: PackedStringArray = []
		for key in simulator.state.variables:
			vars.append("%s = %s" % [key, str(simulator.state.scope._get(key))])
		parts.append("[b]Variables:[/b]\n%s" % "\n".join(vars) if vars.size() else "[b]Variables:[/b] (empty)")
	if !simulator.recessive.is_empty():
		var entries: PackedStringArray = []
		for key in simulator.recessive:
			entries.append("%s = %s" % [key, str(simulator.recessive[key])])
		parts.append("[b]Recessive:[/b]\n%s" % "\n".join(entries))
	if !simulator.dominant.is_empty():
		var entries: PackedStringArray = []
		for key in simulator.dominant:
			entries.append("%s = %s" % [key, str(simulator.dominant[key])])
		parts.append("[b]Dominant:[/b]\n%s" % "\n".join(entries))
	text = "\n\n".join(parts)
