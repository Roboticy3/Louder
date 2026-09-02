extends Node

enum ClientProperties {
	username = 0,
	authority,
	character,
}

var client_data:Dictionary[int, Dictionary]
var usernames:Dictionary[String, int]

@rpc("any_peer", "call_local", "reliable")
func update_client_data(data:Dictionary[ClientProperties, Variant]):
	if !multiplayer.is_server():
		return
	
	var authority := multiplayer.get_remote_sender_id()
	
	var username:String = data.get(ClientProperties.username, "")
	var given_authority = data.get(ClientProperties.authority, null)
	
	if username.is_empty():
		#notify calling authority that username is bad
		pass
	
	if given_authority is int and given_authority != authority:
		#notify calling authority that given authority is invalid
		pass
