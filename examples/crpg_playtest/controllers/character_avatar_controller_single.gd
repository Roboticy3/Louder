extends Node

@export var character_avatar_group := &"character_avatar"
@export var character_data:CharacterData

@export var camera_rig:CameraRigSimple 

var locate_timer:Timer
signal controlled_avatar_changed(to: CharacterAvatar)
var controlled_avatar: CharacterAvatar :
	set(new_controlled_avatar):
		if new_controlled_avatar != controlled_avatar:
			controlled_avatar = new_controlled_avatar
			
			if !new_controlled_avatar:
				locate_timer.start()
			else:
				camera_rig.target = controlled_avatar.body
				print("located character avatar ", controlled_avatar)
				locate_timer.stop()
				
			
			controlled_avatar_changed.emit(new_controlled_avatar)

func locate_character() -> void:
	if !is_inside_tree():
		return
	
	for c in get_tree().get_nodes_in_group(character_avatar_group):
		if c is CharacterAvatar:
			if c.character_data == character_data:
				controlled_avatar = c
	
	controlled_avatar = null

func _ready() -> void:
	locate_timer = Timer.new()
	locate_timer.one_shot = false
	locate_timer.wait_time = 1.0
	locate_timer.timeout.connect(locate_character)
	add_child(locate_timer)
	
	locate_character()
