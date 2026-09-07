extends Node

@export var trigger_listener_group := &"trigger_listener"

func _on_ps_simulator_callback(args: PackedStringArray) -> void:
	if args.is_empty(): return
	
	match args[0]:
		"trigger":
			_on_trigger(args)

func _on_trigger(args:PackedStringArray) -> void:
	if args.size() < 2: 
		push_error("Expected two arguments for callback 'trigger': ", " ".join(args))
		return
	
	get_tree().call_group(trigger_listener_group, &"_on_trigger", args[1])
