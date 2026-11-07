extends Control

## A server-side log of every event the engine has produced.
##
## The engine's event queue is a work in progress: the server drains it as it
## relays events to the other peers, so anything read from the queue disappears
## once it has been handled. The trace is the history instead, which is what this
## log reads, so it can show the events that were only ever meant to be sent to
## another client as well as the ones the server acted on itself.
##
## Only the server has an engine to trace. A client hides this node rather than
## showing an empty log it could never fill.

@export var container: VBoxContainer
@export var max_logs := 20

# How far into the trace the next label comes from. The trace only ever grows,
# so this is all that is needed to avoid printing the same event twice.
var _next_trace_index := 0
var _previous_event: Dictionary = {}


func _ready() -> void:
	# Only the peer that actually runs the engine should keep this node: the
	# host of a lobby, or a lone offline peer. A client in a lobby has no trace
	# of its own, and `multiplayer.is_server()` alone is true even with no peer.
	if not (not Network.is_in_lobby() or Network.is_host()):
		queue_free()
		return

	if container == null:
		container = get_node_or_null("VBoxContainer") as VBoxContainer
	if container == null:
		push_error("A LouderEventLog has no container to put labels in.")
		set_process(false)


func _process(_delta: float) -> void:
	# Re-check on every frame: a peer can start as an offline host and join a
	# lobby later, and a client never has a trace of its own.
	if not (not Network.is_in_lobby() or Network.is_host()):
		queue_free()
		return

	var traced := LouderContextGlobal.context.get_traced_event_count()

	# A trace that got shorter was cleared, so what is on screen is stale.
	if traced < _next_trace_index:
		_clear_labels()
		_next_trace_index = 0
		_previous_event = {}

	while _next_trace_index < traced:
		var event: LouderEvent = LouderContextGlobal.context.get_traced_event(_next_trace_index)
		_next_trace_index += 1
		if event == null:
			continue
		_add_label(event.to_dictionary_resolved(LouderContextGlobal.context), event.get_conversation())

	while container.get_child_count() > max_logs:
		var oldest := container.get_child(0)
		container.remove_child(oldest)
		oldest.queue_free()
	
	#size.y = container.size.y


func _flatten(event: Dictionary, prefix: String = "") -> Dictionary:
	var out := {}
	for key in event.keys():
		var path := str(key) if prefix == "" else prefix + "." + str(key)
		var value = event[key]
		if value is Dictionary:
			var nested := _flatten(value, path)
			for nested_key in nested.keys():
				out[nested_key] = nested[nested_key]
		else:
			out[path] = value
	return out


func _add_label(event: Dictionary, conversation: int = -1) -> void:
	var current := _flatten(event)
	var previous := {}
	if not _previous_event.is_empty():
		previous = _flatten(_previous_event)

	var lines := []
	for key in current.keys():
		var is_load_failure_key: bool = key.begins_with("load_failure.")
		var is_load_error_code_key: bool = key == "load_failure.error" or key.ends_with(".error")
		# Load errors are reported through the trace one record at a time, and
		# the godot error log prints their position and code for every record.
		# The log only prints changed fields by default, so a run of records
		# that fail with the same position or code would otherwise drop them;
		# force every load-failure field through.
		if is_load_failure_key or not previous.has(key) or str(previous[key]) != str(current[key]):
			var value = current[key]
			if _is_hub_position_key(key):
				value = _format_hub_position(conversation, int(value))
			elif is_load_error_code_key:
				value = LouderLoader.error_name(int(value))
			lines.append("%s: %s" % [key, value])

	if lines.is_empty():
		lines.append("(no changes)")

	var label := Label.new()
	label.text = "\n".join(lines)
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	container.add_child(label)

	_previous_event = event.duplicate(true)


func _is_hub_position_key(key: String) -> bool:
	return key == "position" or key.ends_with(".position") or key.ends_with("_position")


func _truncate_hub_text(text: String) -> String:
	var words := text.replace("\n", " ").replace("\t", " ").split(" ", false)
	if words.size() <= 5:
		return text.strip_edges()
	var snippet := " ".join(words.slice(0, 5))
	return snippet + "..."


func _format_hub_position(conversation: int, position: int) -> String:
	if position < 0:
		return str(position)
	var text := ""
	if conversation >= 0:
		text = LouderContextGlobal.get_line_text(conversation, position)
	var snippet := _truncate_hub_text(text)
	if snippet == "":
		return str(position)
	return "%d %s" % [position, snippet]


func _clear_labels() -> void:
	for child in container.get_children():
		container.remove_child(child)
		child.queue_free()
	_previous_event = {}
