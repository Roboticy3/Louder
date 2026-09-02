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
		SpawnMode.SPAWN_ON_CHARACTER_SELECTED:
			ClientDataLayer.character_selected.connect(_on_spawn_trigger)
	
	add_spawnable_scene(spawnable_packed.resource_path)

var server_highest_id := 0
var server_spawn_registry:Dictionary[int, Node]
func _on_spawn_trigger(authority:int, _dump=null) -> void:
	if !multiplayer.is_server(): return
	
	var spawn_root = get_node(get("spawn_path"))
	var spawnable_instance := spawnable_packed.instantiate()
	
	if authority in server_spawn_registry:
		var marked := server_spawn_registry[authority]
		if is_instance_valid(marked):
			marked.process_mode = Node.PROCESS_MODE_DISABLED
			marked.queue_free()
	
	spawnable_instance.set_meta("authority", authority)
	server_spawn_registry[authority] = spawnable_instance
	
	spawnable_instance.name = str(server_highest_id)
	server_highest_id += 1
	
	spawn_root.add_child.call_deferred(spawnable_instance)
