extends Node


func _on_ps_simulator_callback(args: PackedStringArray) -> void:
	if args.is_empty(): return
	
	match args[0]:
		"trigger":
			_on_trigger(args)

func _on_trigger(args:PackedStringArray) -> void:
	if args.size() < 2: 
		push_error("Expected two arguments for callback 'trigger': ", " ".join(args))
		return
	
	match args[1]:
		"reka_appears": _on_reka_appears.rpc()

@rpc("authority", "call_local", "reliable")
func _on_reka_appears():
	pass
