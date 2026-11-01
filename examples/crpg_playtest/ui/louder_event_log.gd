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
	if not multiplayer.is_server():
		queue_free()
		return

	if container == null:
		container = get_node_or_null("VBoxContainer") as VBoxContainer
	if container == null:
		push_error("A LouderEventLog has no container to put labels in.")
		set_process(false)


func _process(_delta: float) -> void:
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
		_add_label(event.to_dictionary_resolved(LouderContextGlobal.context))

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


func _add_label(event: Dictionary) -> void:
	var current := _flatten(event)
	var previous := {}
	if not _previous_event.is_empty():
		previous = _flatten(_previous_event)

	var lines := []
	for key in current.keys():
		if not previous.has(key) or str(previous[key]) != str(current[key]):
			lines.append("%s: %s" % [key, current[key]])

	if lines.is_empty():
		lines.append("(no changes)")

	var label := Label.new()
	label.text = "\n".join(lines)
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	label.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	container.add_child(label)

	_previous_event = event.duplicate(true)


func _clear_labels() -> void:
	for child in container.get_children():
		container.remove_child(child)
		child.queue_free()
	_previous_event = {}
