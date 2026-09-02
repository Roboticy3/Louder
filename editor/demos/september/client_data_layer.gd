extends Node

enum ClientProperties {
	authority = 0,
	character,
}

enum DataError {
	INVALID_CHARACTER = 0,
	CHARACTER_ALREADY_IN_USE,
	INVALID_AUTHORITY
}

signal data_error(authority: int, error: DataError)

signal data_updated(authority: int, data: Dictionary)

var client_data:Dictionary[int, Dictionary]
var characters:Dictionary[String, int]

@rpc("authority", "call_local", "reliable")
func sync_client_data(authority: int, data: Dictionary, new_characters:Dictionary):
	client_data[authority] = data
	characters = new_characters
	data_updated.emit(authority, data)

@rpc("authority", "call_local", "reliable")
func sync_data_error(error: DataError):
	data_error.emit(error)

@rpc("any_peer", "call_local", "reliable")
func update_client_data(data:Dictionary):
	if !multiplayer.is_server():
		return
	
	var authority := multiplayer.get_remote_sender_id()
	
	var new_character:String = data.get(ClientProperties.character, "")
	var given_authority = data.get(ClientProperties.authority, null)
	
	if new_character:
		var c_authority = characters.get(new_character)
		if c_authority != null and c_authority != authority:
			sync_data_error.rpc_id(authority, DataError.CHARACTER_ALREADY_IN_USE)
			return
	
	if given_authority is int and given_authority != authority:
		sync_data_error.rpc_id(authority,  DataError.INVALID_AUTHORITY)
		return
	
	if !client_data.has(authority):
		client_data[authority] = data
	else:
		var old_character = client_data[authority].get(ClientProperties.character, "")
		if !old_character.is_empty():
			characters.erase(old_character)
		client_data[authority].merge(data, true)
	
	var confirmed_character = client_data[authority].get(ClientProperties.character, "")
	if !confirmed_character.is_empty():
		characters[confirmed_character] = authority
	
	sync_client_data.rpc(authority, data, characters)

func is_character_available(character:String):
	return !character.is_empty() and !(character in characters)
