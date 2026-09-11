extends CanvasLayer

@export var bot_button:BaseButton
@export var override_button:BaseButton
@export var backdrop:Window
@export var simulator:PSSimulator
@export var config:PSConfig

@export var exclude := []

@export var bot_packed:PackedScene
var bot_summoned := false

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	ClientDataLayer.data_updated.connect(_check_data)
	
	override_button.pressed.connect(hide_backdrop)
	bot_button.pressed.connect(_summon_bot)
	
	if !multiplayer.is_server():
		bot_button.visible = false
	
	_check_data()
	
	get_window().size_changed.connect(position_backdrop)
	position_backdrop()

func position_backdrop():
	backdrop.size = get_window().size
	backdrop.position = Vector2.ZERO

func show_backdrop():
	backdrop.size = get_window().size
	backdrop.position = Vector2.ZERO
	backdrop.visible = true
	
func hide_backdrop():
	backdrop.visible = false

func _check_data(_a=null, _d=null):
	var cd := ClientDataLayer.client_data
	if !(multiplayer.get_unique_id() in cd):
		hide_backdrop()
	elif cd.size() < 2 and !bot_summoned and !override_button.button_pressed:
		show_backdrop()
	else:
		hide_backdrop()

func _get_unpossessed_ownable_speakers() -> PackedStringArray:
	var possessed: PackedStringArray
	for controller in get_tree().get_nodes_in_group(&"ps_controller"):
		if controller is PSController:
			possessed.append_array(controller.speak_as)
	
	var unpossessed := PackedStringArray()
	for speaker in config.list_ownable_speakers():
		if !(speaker in possessed) and !(speaker in exclude):
			unpossessed.append(speaker)
	return unpossessed

func _summon_bot():
	if !multiplayer.is_server():
		return
	
	if !simulator or !config:
		return
	
	#summon a bot for each ownable speaker not already possessed by a controller
	#"bot" is a PSController with a timer attached to automatically make choices
	#location of bot in the scene tree shouldn't matter. As long as it's 
	#speak_as, listen_to, and simulator properties are set correctly.
	for speaker in _get_unpossessed_ownable_speakers():
		var bot := bot_packed.instantiate() as PSController
		bot.speak_as = PackedStringArray([speaker])
		#listen_to is filled out using the speak_as speakers' relationships in config
		bot.listen_to = PackedStringArray(config.get_relationships(speaker))
		bot.simulator = simulator
		bot.config = config
		
		add_child(bot)
	
	bot_summoned = true
	hide_backdrop()
