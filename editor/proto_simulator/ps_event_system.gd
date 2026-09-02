extends Node

#send a choice to the server
#the server then uses the `authority` and `conversation_path` to find a matching
#controller.
#if found, the resulting controller makes the choice, if available
#the server computes the move on that controller, then repeats the update down
#to all peers.
@rpc("any_peer", "call_local", "reliable")
func send_choice_to_server(authority: int, conversation_path: String, choice: String) -> void:
	if !multiplayer.is_server():
		return
