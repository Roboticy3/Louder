extends Control

#when changed, change the AudioServer bus level to match
@export var slider:Range
@export var bus_label:Label

var bus_name: StringName

func _ready() -> void:
	bus_name = name
	bus_label.text = bus_name
	slider.min_value = -80.0
	slider.max_value = 0.0
	slider.step = 0.5
	slider.value = Settings.get_bus_volume(bus_name)
	slider.value_changed.connect(_on_slider_value_changed)

func _on_slider_value_changed(value: float) -> void:
	Settings.set_bus_volume(bus_name, value)
