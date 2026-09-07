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
	
	trigger_rpc.rpc(args[1])

@rpc("authority", "call_local", "reliable")
func trigger_rpc(trigger:String) -> void:
	get_tree().call_group(trigger_listener_group, &"_on_trigger", trigger)
