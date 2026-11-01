extends Node

var conversation_path:String
var config_path:String

const ARGS := [
	"--open-project="
]

func _ready() -> void:
	var args := OS.get_cmdline_user_args()
	for a in args:
		if a.begins_with(ARGS[0]):
			conversation_path = a.substr(ARGS[0].length())
			pass
