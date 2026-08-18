extends CodeEdit

@export var simulator:PSSimulator
@export var output_stream:CodeEdit

enum CommandError {
	OK = 0,
	SYNTAX = 1,
	UNRECOGNIZED,
	SIMULATOR,
	INDEX,
}

var busy := false
func flush():
	if busy:
		return
	busy = true
	
	var error:CommandError = CommandError.OK
	if text.begins_with("/"):
		var command := text.trim_prefix("/").split(" ", false)
		if command.is_empty(): error = CommandError.UNRECOGNIZED
		else: match command[0]:
			"warp": error = warp(command)
			"setspeakers": error = setspeakers(command)
			_: error = CommandError.UNRECOGNIZED
	
	match error:
		CommandError.SYNTAX: output_stream.text += "command \"" + text + "\" exited with syntax error.\n"
		CommandError.UNRECOGNIZED: output_stream.text += "command \"" + text + "\" was not recognized.\n"
		CommandError.SIMULATOR: output_stream.text += "simulator could not process command \"" + text + "\". Exited with simulator error.\n" 
		CommandError.SIMULATOR: output_stream.text += "command \"" + text + "\" caused an index exception.\n"
		
	await get_tree().process_frame
	text = ""
	busy = false

func warp(command:Array) -> CommandError:
	if command.size() < 2: return CommandError.SYNTAX
	
	if simulator.warp(int(command[1])):
		output_stream.text += "warped to position " + str(command[1]) + "\n"
		simulator.choice_made.emit()
		return CommandError.OK
	else:
		output_stream.text += "failed to warp to position " + str(command[1]) + "\n"
		return CommandError.SIMULATOR

func setspeakers(command:Array) -> CommandError:
	if command.size() < 3: return CommandError.SYNTAX
	
	if !str(command[1]).is_valid_int(): return CommandError.SYNTAX
	
	var vi = int(command[1])
	var views := get_tree().get_nodes_in_group(&"ps_conversation_view")
	if vi < 0 || vi >= views.size(): return CommandError.INDEX
	
	var speakers = views[vi].get("my_speakers")
	views[vi].set("my_speakers", command.slice(2))
	views[vi].call("update")
	output_stream.text += "changed my_speakers on viewer " + str(vi) + " from " + str(speakers) + " to " + str(command.slice(2)) + "\n"
	
	return CommandError.OK

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _gui_input(event:InputEvent) -> void:
	if event.is_action_pressed("command_send"):
		flush()
