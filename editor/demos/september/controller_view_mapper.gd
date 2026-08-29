extends Node

@export var view:PackedScene
@export var view_root:Node
@export var controller_root:Node

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	var authority := multiplayer.multiplayer_peer.get_unique_id()
	set_multiplayer_authority(authority)
	controller_root.child_entered_tree.connect(map_controller)

func map_controller(n:Node):
	if n.get_multiplayer_authority() != get_multiplayer_authority():
		return
	
	var controller = n.get("controller")
	if controller is PSController:
		var view_instance := view.instantiate()
		view_instance.set("controller", controller)
		view_instance.set_multiplayer_authority(get_multiplayer_authority(), true)
		view_root.add_child.call_deferred(view_instance)
