extends CodeEdit

@export var simulator:PSSimulator
@export var output_stream:CodeEdit

enum CommandError {
	OK = 0,
	SYNTAX = 1,
	UNRECOGNIZED,
	SIMULATOR
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
			_: error = CommandError.UNRECOGNIZED
	
	match error:
		CommandError.SYNTAX: output_stream.text += "command \"" + text + "\" exited with syntax error.\n"
		CommandError.UNRECOGNIZED: output_stream.text += "command \"" + text + "\" was not recognized.\n"
		CommandError.SIMULATOR: output_stream.text += "simulator could not process command \"" + text + "\". Exited with simulator error.\n" 
	
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

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _gui_input(event:InputEvent) -> void:
	if event.is_action_pressed("command_send"):
		flush()
