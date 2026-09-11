extends Node

@export var host_ip_label:Label
@export var host_port_label:Label

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	update()
	NetworkLayer.host_details_changed.connect(update)

func update():
	host_ip_label.text = str(NetworkLayer.get_host_ip())
	host_port_label.text = str(NetworkLayer.get_host_port())
