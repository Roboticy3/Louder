extends Node

## PROBE: what does the multiplayer layer actually do today?

const CONFIG := "res://configs/config.json"


func _ready() -> void:
	print("--- probe start ---")
	print("authority: ", LouderContextGlobal.multiplayer.is_server())

	var config_result: int = LouderContextGlobal.load_conversation_config_file(CONFIG)
	print("config load: ", config_result)

	var conversation: int = LouderContextGlobal.context.make_name(&"probe")
	var document := JSON.stringify([
		{"type": "hub", "conversation_id": "0", "tags": "start", "text": "start"},
		{"type": "hub", "conversation_id": "1", "speaker": "Benjamin", "text": "ben says"},
		{"type": "hub", "conversation_id": "2", "speaker": "Geno", "text": "geno says"},
		{"type": "connection", "from_id": "0", "from_port": 0, "to_id": "1"},
		{"type": "connection", "from_id": "0", "from_port": 1, "to_id": "2"},
	])
	var load_result: int = LouderContextGlobal.context.load_conversation(conversation, document)
	print("conversation load: ", load_result)

	LouderContextGlobal.actor_added.connect(func(c: int, a: int) -> void: print("  signal actor_added c=%d a=%d" % [c, a]))
	LouderContextGlobal.motion.connect(func(c: int, a: int, t: int, bg: int, s: String) -> void: print("  signal motion c=%d a=%d t=%d bg=%d s=%s" % [c, a, t, bg, s]))
	LouderContextGlobal.callback_raised.connect(func(cb: String) -> void: print("  signal callback %s" % cb))

	var benjamin := CharacterData.new()
	benjamin.name = "Benjamin"
	var geno := CharacterData.new()
	geno.name = "Geno"
	var debug_character := CharacterData.make_debug()

	print("benjamin speakers: ", benjamin.get_speaker_names(), " flags: ", benjamin.get_actor_flags())
	print("debug speakers: ", debug_character.get_speaker_names(), " flags: ", debug_character.get_actor_flags())

	var actor_a: int = LouderContextGlobal.connect_actor(conversation, ["start"], benjamin.get_actor_flags(), benjamin.get_speaker_names())
	print("peer A actor: ", actor_a)

	var actor_b: int = LouderContextGlobal.connect_actor(conversation, ["start"], geno.get_actor_flags(), geno.get_speaker_names())
	print("peer B actor: ", actor_b)

	await get_tree().process_frame
	await get_tree().process_frame

	for actor in [actor_a, actor_b]:
		var info: Actor = LouderContextGlobal.context.get_actor(conversation, actor)
		print("actor %d: pos=%d channel=%s flags=%d speak_as=%s" % [
			actor, info.get_position(), LouderContextGlobal.resolve_name(info.get_channel()),
			info.get_flags(), info.speak_as()])
		print("   choices at 0: ", LouderContextGlobal.get_available_choice_indices(conversation, actor))
		for index in LouderContextGlobal.get_available_choice_indices(conversation, actor):
			print("     index %d -> hub %d text %s" % [
				index,
				LouderContextGlobal.get_choice_text(conversation, 0, actor, index),
				LouderContextGlobal.context.get_hub_choice_link(conversation, 0, actor, index)])

	print("--- probe end ---")
