extends MultiplayerSpawner

enum SpawnMode {
	SPAWN_ON_PEER_CONNECTED = 0,
	SPAWN_ON_CHARACTER_SELECTED,
}
@export var spawn_mode:SpawnMode

@export var spawnable_packed:PackedScene

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	match spawn_mode:
		SpawnMode.SPAWN_ON_PEER_CONNECTED:
			multiplayer.peer_connected.connect(_on_spawn_trigger)
			NetworkLayer.server_created.connect(_on_spawn_trigger.bind(1))
		SpawnMode.SPAWN_ON_CHARACTER_SELECTED:
			ClientDataLayer.character_selected.connect(_on_spawn_trigger)
		
	
	var spawn_root := get_node(get("spawn_path"))
	spawn_root.child_entered_tree.connect(_on_spawn_root_child_entered_tree)
	spawned.connect(_on_remote_peer_spawned)
	
	add_spawnable_scene(spawnable_packed.resource_path)

var server_highest_id := 0
var server_spawn_registry:Dictionary[int, Node]
func _on_spawn_trigger(authority:int, _dump=null) -> void:
	if !multiplayer.is_server(): return
	
	var spawn_root := get_node(get("spawn_path"))
	var spawnable_instance := spawnable_packed.instantiate()
	
	if authority in server_spawn_registry:
		var marked := server_spawn_registry[authority]
		if is_instance_valid(marked):
			marked.process_mode = Node.PROCESS_MODE_DISABLED
			spawn_root.remove_child(marked)
			marked.queue_free()
	
	spawnable_instance.set("authority", authority)
	server_spawn_registry[authority] = spawnable_instance
	
	spawnable_instance.name = str(authority)
	server_highest_id += 1
	
	spawn_root.add_child.call_deferred(spawnable_instance)

func _on_spawn_root_child_entered_tree(node:Node):
	if int(node.name) != 0 and multiplayer.is_server():
		_on_remote_peer_spawned(node)

func _on_remote_peer_spawned(spawnable_instance:Node):
	spawnable_instance.set_multiplayer_authority.call_deferred(int(spawnable_instance.name))
