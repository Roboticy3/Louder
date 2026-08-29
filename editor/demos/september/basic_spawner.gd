extends MultiplayerSpawner 

@export var server_listener_group := &"server_listener"
@export var spawnable_packed:PackedScene

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	for authority in NetworkLayer.client_data:
		_on_peer_connected(authority)
	
	add_spawnable_scene(spawnable_packed.resource_path)
	add_to_group(server_listener_group)
	multiplayer.peer_connected.connect(_on_peer_connected)

func _on_peer_connected(authority:int) -> void:
	if !multiplayer.is_server(): return
	
	var spawn_root = get_node(spawn_path)
	var spawnable_instance := spawnable_packed.instantiate()
	
	# Name the entry after the peer id instead of using an incrementing index.
	# Node names are always replicated to remote peers, but the multiplayer
	# authority set here is not reliably propagated (Godot issue #75067). Each
	# replica recovers the correct authority from its own name in _enter_tree.
	spawnable_instance.name = str(authority)
	spawn_root.add_child.call_deferred(spawnable_instance)

func _on_server_created(_port:int) -> void:
	_on_peer_connected(get_multiplayer_authority())
