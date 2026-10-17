extends Node

var context := LouderContext.new()

func load_conversation_file(file_path) -> int:
	var conversation_name := context.make_name(file_path)
	var fa := FileAccess.open(file_path, FileAccess.READ)
	if FileAccess.get_open_error() != OK:
		push_error("Failed to open conversation ", file_path)
		return -1
	var text := fa.get_as_text()
	context.load_conversation(conversation_name, text)
	return conversation_name

func load_conversation_config_file(file_path) -> int:
	var fa := FileAccess.open(file_path, FileAccess.READ)
	if FileAccess.get_open_error() != OK:
		push_error("Failed to open config ", file_path)
		return -1
	var text := fa.get_as_text()
	context.load_conversation_config(text)
	return 0

func add_actor(conversation: int, start_tags:Array[String] , flags: int, configured_speakers:Array[String], configured_listen_to:Array[String] = []) -> int:
	var start := context.next_hub_from_tags(conversation, start_tags.map(context.make_name), false, -1)
	var speaker := configured_speakers.map(context.make_name)
	var listen_to := configured_listen_to.map(context.make_name)
	return context.call("add_actor", conversation, start, flags, speaker, listen_to)
