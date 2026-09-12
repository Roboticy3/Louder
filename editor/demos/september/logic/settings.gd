extends Node

const SAVE_PATH := "user://settings.json"
const SAVE_VERSION := 1

var bus_volumes := {}
var keybinds := {}

const MENU := preload("res://demos/september/ui/settings_menu.tscn")

func _ready() -> void:
	load_settings()
	add_child(MENU.instantiate())


func load_settings() -> void:
	bus_volumes.clear()
	keybinds.clear()
	if not FileAccess.file_exists(SAVE_PATH):
		return
	var file := FileAccess.open(SAVE_PATH, FileAccess.READ)
	if file == null:
		return
	var data: Variant = JSON.parse_string(file.get_as_text())
	file.close()
	if not data is Dictionary or data.get("version", 0) != SAVE_VERSION:
		return
	var buses: Dictionary = data.get("buses", {})
	for bus_name in buses:
		var db: float = buses[bus_name]
		bus_volumes[StringName(bus_name)] = db
		_apply_bus_volume(StringName(bus_name), db)
	var binds: Dictionary = data.get("keybinds", {})
	for action in binds:
		var event := _event_from_dict(binds[action])
		if event:
			keybinds[StringName(action)] = event
			_apply_keybind(StringName(action), event)


func save_settings() -> void:
	var buses := {}
	for bus_name in bus_volumes:
		buses[String(bus_name)] = bus_volumes[bus_name]
	var binds := {}
	for action in keybinds:
		binds[String(action)] = _event_to_dict(keybinds[action])
	var data := {
		"version": SAVE_VERSION,
		"buses": buses,
		"keybinds": binds,
	}
	var file := FileAccess.open(SAVE_PATH, FileAccess.WRITE)
	if file == null:
		return
	file.store_string(JSON.stringify(data, "\t"))
	file.close()


func get_bus_volume(bus_name: StringName) -> float:
	return bus_volumes.get(bus_name, 0.0)


func set_bus_volume(bus_name: StringName, volume_db: float) -> void:
	bus_volumes[bus_name] = volume_db
	_apply_bus_volume(bus_name, volume_db)
	save_settings()


func _apply_bus_volume(bus_name: StringName, volume_db: float) -> void:
	var index := AudioServer.get_bus_index(bus_name)
	if index >= 0:
		AudioServer.set_bus_volume_db(index, volume_db)


func get_bind_text(action: StringName) -> String:
	var event := get_action_event(action)
	if event:
		return event.as_text()
	return "Unbound"


func get_action_event(action: StringName) -> InputEvent:
	if action in keybinds:
		return keybinds[action]
	var events := InputMap.action_get_events(action)
	if events.is_empty():
		return null
	return events[0]


func bind_action(action: StringName, event: InputEvent) -> void:
	keybinds[action] = event
	_apply_keybind(action, event)
	save_settings()


func _apply_keybind(action: StringName, event: InputEvent) -> void:
	InputMap.action_erase_events(action)
	InputMap.action_add_event(action, event)


func _event_to_dict(event: InputEvent) -> Dictionary:
	var dict := {}
	if event is InputEventKey:
		dict["type"] = "key"
		dict["keycode"] = event.keycode
		dict["physical_keycode"] = event.physical_keycode
		dict["unicode"] = event.unicode
		dict["shift"] = event.shift_pressed
		dict["ctrl"] = event.ctrl_pressed
		dict["alt"] = event.alt_pressed
		dict["meta"] = event.meta_pressed
	elif event is InputEventMouseButton:
		dict["type"] = "mouse"
		dict["button_index"] = event.button_index
	elif event is InputEventJoypadButton:
		dict["type"] = "joypad_button"
		dict["button_index"] = event.button_index
	dict["device"] = event.device
	return dict


func _event_from_dict(dict: Variant) -> InputEvent:
	if not dict is Dictionary:
		return null
	match dict.get("type", ""):
		"key":
			var event := InputEventKey.new()
			event.keycode = dict.get("keycode", 0)
			event.physical_keycode = dict.get("physical_keycode", 0)
			event.unicode = dict.get("unicode", 0)
			event.shift_pressed = dict.get("shift", false)
			event.ctrl_pressed = dict.get("ctrl", false)
			event.alt_pressed = dict.get("alt", false)
			event.meta_pressed = dict.get("meta", false)
			event.device = dict.get("device", 0)
			return event
		"mouse":
			var event := InputEventMouseButton.new()
			event.button_index = dict.get("button_index", MOUSE_BUTTON_LEFT)
			event.device = dict.get("device", 0)
			return event
		"joypad_button":
			var event := InputEventJoypadButton.new()
			event.button_index = dict.get("button_index", 0)
			event.device = dict.get("device", 0)
			return event
	return null
