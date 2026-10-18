extends SceneTree

## Runs test scripts headlessly, one after another.
##
## Godot only runs the script given to `--script` when that script extends
## SceneTree or MainLoop, which rules out the plain Node scripts tests are
## written as, so something has to start them. This is that something: it adds
## each test script to the tree, gives it a few frames, and moves on to the next.
##
##     godot --headless --path . --script res://test_runner.gd -- res://test_louder.gd
##
## Several scripts can be named at once, and they run in the order given. Every
## script gets the same frame budget, which `--frames` sets and which defaults
## to DEFAULT_FRAMES. A test that awaits `process_frame`, or that needs more than
## a `_ready` call to finish, asks for more:
##
##     godot --headless --path . --script res://test_runner.gd -- --frames 60 res://test_louder.gd
##
## A script that decides for itself that it has failed calls
## `get_tree().quit(1)`. Godot stops handing frames out the moment any script
## quits, so that exit code stands and the scripts queued behind it never start.
## The runner exits 1 on its own when a script cannot be loaded or is not a Node
## script, and 0 otherwise.

const DEFAULT_FRAMES := 2

var _queue: Array[String] = []
var _current: Node = null
var _budget := DEFAULT_FRAMES
var _frames := 0
var _failures := 0
var _started := false


func _process(_delta: float) -> bool:
	if not _started:
		_started = true
		_read_arguments()

	if _current != null and _frames > 0:
		_frames -= 1
		return false

	if _current != null:
		_current.queue_free()
		_current = null

	while not _queue.is_empty():
		var path: String = _queue.pop_front()
		print("--- %s" % path)
		_current = _begin(path)
		if _current != null:
			_frames = _budget
			return false

	quit(1 if _failures > 0 else 0)
	return true


func _read_arguments() -> void:
	var arguments := OS.get_cmdline_user_args()
	var index := 0
	while index < arguments.size():
		var argument := arguments[index]
		if argument == "--frames":
			index += 1
			if index >= arguments.size():
				push_error("--frames needs a count after it")
				_failures += 1
				break
			var count := arguments[index].to_int()
			if count > 0:
				_budget = count
			else:
				push_error("--frames needs a positive count, got %s" % arguments[index])
				_failures += 1
		elif argument.begins_with("--"):
			push_error("unknown option %s" % argument)
			_failures += 1
		else:
			_queue.append(argument)
		index += 1

	if _queue.is_empty():
		push_error("no test scripts given, expected res:// paths after --")
		_failures += 1


func _begin(path: String) -> Node:
	var script := load(path) as Script
	if script == null:
		push_error("could not load %s" % path)
		_failures += 1
		return null

	var node := script.new() as Node
	if node == null:
		push_error("%s is not a Node script, so it cannot be added to the tree" % path)
		_failures += 1
		return null

	node.name = path.get_file().get_basename()
	root.add_child(node)
	return node