extends Node

## Checks that every method the extension registers is reachable by its own name.
##
## The bug this guards against is narrow and easy to reintroduce: godot-rust
## reads #[func(rename = "...")] as a token stream, so the string literal
## arrives carrying its own quotes and the method ends up registered under
## "\"add_actor\"". GDScript still reaches it, because a call to a name it does
## not know falls back to a dynamic one, but the script editor shows it in
## quotes, completion never offers it, and nothing type-checks against it. A
## quoted name anywhere in this list is that bug.

const CLASSES := [
	"LouderContext",
	"LouderEvent",
	"LouderAsk",
	"Motion",
	"ChoiceAppearance",
	"AdditionalDataChange",
	"ConditionOutcome",
	"LoadFailure",
	"Hub",
	"Choice",
	"Actor",
	"ActorState",
	"Speaker",
	"Conversation",
	"HomogeneousObject",
	"EvaluationScope",
	"Evaluator",
	"ParseError",
	"LouderNameContent",
	"LouderLoader",
	"LouderJson",
	"LouderQuery",
	"LouderDialogueRules",
	"LouderStatus",
	"Simulator",
]

# Object's own methods, present on every class, and none of them ours.
const INHERITED := [
	"call", "call_deferred", "callv", "free", "get_class", "get_instance_id",
	"get_meta", "get_method_list", "get_script", "has_meta", "has_method",
	"is_class", "notification", "to_string", "set_script", "set_meta",
	"remove_meta", "connect", "disconnect", "is_connected", "emit_signal",
	"has_signal", "get_signal_list", "get_signal_connection_list",
	"add_user_signal", "remove_user_signal", "get_signal_connection_list",
	"set_deferred", "get_property_list", "property_can_revert",
	"property_get_revert", "cancel_free", "is_queued_for_deletion",
	"reference", "unreference", "get_reference_count", "init_ref", "_init",
	"_to_string", "_notification", "_set", "_get", "_get_property_list",
	"_validate_property", "_property_can_revert", "_property_get_revert",
	"_iter_init", "_iter_next", "_iter_get", "get", "set", "set_indexed",
	"get_indexed", "notification", "tr", "tr_n", "get_translation_domain",
	"set_message_translation", "can_translate_messages",
]

var _failures := 0


func _ready() -> void:
	var checked := 0
	for class_id in CLASSES:
		if not ClassDB.class_exists(class_id):
			_fail("class %s is not registered" % class_id)
			continue
		checked += _check_names(class_id, ClassDB.instantiate(class_id))

	print("PASS %d registered names are all plain identifiers" % checked)

	if _failures > 0:
		get_tree().quit(1)


## Every name the class brought with it has to be a plain identifier.
##
## `create` is the one static method a script is expected to call on the class
## itself rather than on an instance, so an instance list does not cover it.
func _check_names(class_id: String, instance: Object) -> int:
	var names := []
	for method in instance.get_method_list():
		names.append(method.name)

	for entry in ClassDB.class_get_method_list(class_id, true):
		names.append(entry.name)

	var seen := {}
	var checked := 0
	for name in names:
		if seen.has(name) or INHERITED.has(name):
			continue
		seen[name] = true
		checked += 1
		if name.begins_with('"') or name.ends_with('"'):
			_fail("%s.%s is registered with quotes around it, so only call() can reach it" % [class_id, name])
	return checked


func _fail(message: String) -> void:
	print("FAIL ", message)
	_failures += 1