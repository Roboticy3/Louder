extends CanvasLayer

@export var bot_button:BaseButton
@export var override_button:BaseButton
@export var simulator:PSSimulator
@export var config:PSConfig

@export var bot_packed:PackedScene = load("res://demos/september/specific/bot.tscn")
var bot_summoned := false

# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	ClientDataLayer.data_updated.connect(_check_data)
	
	override_button.pressed.connect(hide)
	bot_button.pressed.connect(_summon_bot)

func _check_data(_a, _d):
	var cd := ClientDataLayer.client_data
	if cd.size() < 2 and !bot_summoned:
		show()
	else:
		hide()

func _get_missing_character() -> String:
	var cd := ClientDataLayer.client_data
	if cd.size() < 2 and !bot_summoned:
		return "TODO" #Geno if Reka is present, Reka if Geno is present
	else:
		return ""

func _summon_bot():
	if !multiplayer.is_server():
		return
	
	#speak_as = [bot_play_as] later
	var bot_play_as := _get_missing_character()
	if bot_play_as.is_empty():
		return
	
	if !simulator or !config:
		return
	
	#summon a bot and connect it to the simulator
	#"bot" is a PSController with a timer attached to automatically make choices
	#location of bot in the scene tree shouldn't matter. As long as it's 
	#speak_as, listen_to, and simulator properties are set correctly.
	
	#listen_to is filled out using the speak_as speakers' relationships in config
	
	bot_summoned = true
