extends Node #extending MultiplayerSpawner seems to cause a bug(?) here. 2spooky4

@export var server_listener_group := &"server_listener"
@export var spawnable_packed:PackedScene

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	add_to_group(server_listener_group)
	multiplayer.peer_connected.connect(_on_peer_connected)
	

func _on_peer_connected(authority:int) -> void:
	if !multiplayer.is_server(): return
	
	var spawn_root = get_node(get("spawn_path"))
	var spawnable_instance := spawnable_packed.instantiate()
	
	spawn_root.add_child.call_deferred(spawnable_instance)
	spawnable_instance.name = produce_name(spawn_root)
	spawnable_instance.set_multiplayer_authority(authority)

func produce_name(on:Node) -> String:
	if on.get_child_count() == 0:
		return "0"
	
	var last := on.get_child(-1)
	return str(int(last.name) + 1)

func _on_server_created(_port:int) -> void:
	_on_peer_connected(get_multiplayer_authority())
