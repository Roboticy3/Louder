extends Node

@export var host_ip_label:Control
@export var host_port_label:Control

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	update()
	NetworkLayer.host_details_changed.connect(update)

func update():
	host_ip_label.set("text", str(NetworkLayer.get_host_ip()))
	host_port_label.set("text", str(NetworkLayer.get_host_port()))
