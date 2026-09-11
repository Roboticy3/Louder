extends Node

const SAVE_PATH := "user://settings.cfg"
const SAVE_VERSION := 1

var bus_volumes := {}
var keybinds := {}

const SETTINGS_MENU := preload("res://demos/september/ui/settings_menu.tscn")

func _ready() -> void:
	load_settings()
	add_child(SETTINGS_MENU.instantiate())


func load_settings() -> void:
	bus_volumes.clear()
	keybinds.clear()
	var file := FileAccess.open(SAVE_PATH, FileAccess.READ)
	if file == null:
		return
	if file.get_32() != SAVE_VERSION:
		file.close()
		return
	var audio_count := file.get_32()
	for _i in audio_count:
		var bus_name: StringName = file.get_var()
		var volume_db: float = file.get_float()
		bus_volumes[bus_name] = volume_db
		_apply_bus_volume(bus_name, volume_db)
	var bind_count := file.get_32()
	for _i in bind_count:
		var action: StringName = file.get_var()
		var event = file.get_var()
		if !(event is InputEvent):
			continue
		keybinds[action] = event
		_apply_keybind(action, event)
	file.close()


func save_settings() -> void:
	var file := FileAccess.open(SAVE_PATH, FileAccess.WRITE)
	if file == null:
		return
	file.store_32(SAVE_VERSION)
	file.store_32(bus_volumes.size())
	for bus_name in bus_volumes:
		file.store_var(bus_name)
		file.store_float(bus_volumes[bus_name])
	file.store_32(keybinds.size())
	for action in keybinds:
		file.store_var(action)
		file.store_var(keybinds[action])
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
