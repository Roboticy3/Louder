extends RichTextLabel

@export var simulator: PSSimulator

@export var show_conversation := true
@export var show_position := true
@export var show_choices := true
@export var show_variables := true
@export var all_branches := true

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
	
	var branches := simulator.state.keys()
	if !all_branches:
		if "" in branches:
			branches = [""]
		elif !branches.is_empty():
			branches = branches.slice(0, 1)
	
	for branch in branches:
		parts.append("[b]Branch: %s [/b]" % str(branch))
		if show_position:
			parts.append("  [i]Position:[/i] %s" % str(simulator.get_position(branch)))
		if show_choices:
			var raw_choices := simulator.list_choices_data(null, branch)
			var choice_texts: PackedStringArray = []
			for c in raw_choices:
				choice_texts.append("  %s" % str(c.get("from_choice", "?")))
			parts.append("  [i]Choices:[/i]\n%s" % "\n".join(choice_texts) if choice_texts.size() else "[b]Choices:[/b] (none)")
		if show_variables and simulator.state.get(branch):
			var vars: PackedStringArray = []
			for key in simulator.state[branch].variables:
				vars.append("  %s = %s" % [key, str(simulator.state[branch].scope._get(key))])
			parts.append("  [i]Variables:[/i]\n%s" % "\n".join(vars) if vars.size() else "[b]Variables:[/b] (empty)")
	
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
