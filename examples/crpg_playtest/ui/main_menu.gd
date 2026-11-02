class_name MainMenu extends Control

signal host_requested(port: int, dedicated: bool)
signal join_requested(address: String, port: int)

const DEFAULT_ADDRESS := "127.0.0.1"
const DEFAULT_PORT := 7000

@export var address_edit: LineEdit
@export var port_edit: LineEdit
@export var host_button: Button
@export var join_button: Button
@export var dedicated_button: Button
@export var status_label: Label

func _ready() -> void:
	address_edit.text = DEFAULT_ADDRESS
	port_edit.text = str(DEFAULT_PORT)

	host_button.pressed.connect(_on_host_pressed)
	join_button.pressed.connect(_on_join_pressed)
	dedicated_button.pressed.connect(_on_dedicated_pressed)

	_apply_cmdline_args()


func _apply_cmdline_args() -> void:
	
	
	#var args := OS.get_cmdline_user_args()
	#var i := 0
	#var do_host := false
	#var do_join := false
	#var dedicated := false
	#while i < args.size():
		#match args[i]:
			#"--host", "-h":
				#do_host = true
				#i += 1
				#if i < args.size() and args[i].is_valid_int():
					#port_edit.text = args[i]
					#i += 1
			#"--join", "-j":
				#do_join = true
				#i += 1
				#if i < args.size():
					#var target := args[i]
					#if ":" in target:
						#var parts := target.split(":")
						#address_edit.text = parts[0]
						#if parts.size() > 1 and parts[1].is_valid_int():
							#port_edit.text = parts[1]
					#else:
						#address_edit.text = target
						#if i + 1 < args.size() and args[i + 1].is_valid_int():
							#port_edit.text = args[i + 1]
							#i += 1
					#i += 1
			#"--port", "-p":
				#i += 1
				#if i < args.size():
					#port_edit.text = args[i]
					#i += 1
			#"--address", "-a":
				#i += 1
				#if i < args.size():
					#address_edit.text = args[i]
					#i += 1
			#"--dedicated", "-d":
				#dedicated = true
				#i += 1
			#_:
				#i += 1
	#if do_host:
		#call_deferred("_deferred_host", dedicated)
	#elif do_join:
		#call_deferred("_deferred_join")
	
	pass


func _deferred_host(dedicated: bool) -> void:
	host_requested.emit(get_port(), dedicated)


func _deferred_join() -> void:
	join_requested.emit(get_address(), get_port())

func set_status(status: String) -> void:
	status_label.text = status

func get_address() -> String:
	return address_edit.text.strip_edges()

func get_port() -> int:
	return port_edit.text.to_int() if port_edit.text.is_valid_int() else DEFAULT_PORT

func _on_host_pressed() -> void:
	host_requested.emit(get_port(), false)

func _on_join_pressed() -> void:
	join_requested.emit(get_address(), get_port())

func _on_dedicated_pressed() -> void:
	host_requested.emit(get_port(), true)
