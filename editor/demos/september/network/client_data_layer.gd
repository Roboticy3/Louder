extends Node

enum ClientProperties {
	authority = 0,
	character,
	conversation,
}

enum DataError {
	INVALID_CHARACTER = 0,
	CHARACTER_ALREADY_IN_USE,
	INVALID_AUTHORITY
}

signal data_error(error: DataError)

signal data_updated(authority: int, data: Dictionary)
signal character_selected(authority: int, character: String)

var client_data:Dictionary[int, Dictionary]
var characters:Dictionary[String, int]

@rpc("authority", "call_remote", "reliable")
func sync_client_data(authority: int, data: Dictionary, new_characters:Dictionary):
	var old_character := ""
	if authority in client_data:
		old_character = client_data[authority].get(ClientProperties.character)
	
	var new_character = data.get(ClientProperties.character, "")
	
	if authority in client_data:
		client_data[authority].merge(data, true)
	else:
		client_data[authority] = data
	characters = new_characters
	
	data_updated.emit(authority, data)
	if !new_character.is_empty() and new_character != old_character:
		character_selected.emit(authority, new_character)

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
	
	if !new_character.is_empty():
		var c_authority = characters.get(new_character)
		if c_authority != null and c_authority != authority:
			sync_data_error.rpc_id(authority, DataError.CHARACTER_ALREADY_IN_USE)
			return
	
	if given_authority is int and given_authority != authority:
		sync_data_error.rpc_id(authority,  DataError.INVALID_AUTHORITY)
		return
	
	var old_character := ""
	if !client_data.has(authority):
		client_data[authority] = data
		pass
	else:
		old_character = client_data[authority].get(ClientProperties.character, "")
		if !old_character.is_empty():
			characters.erase(old_character)
		client_data[authority].merge(data, true)
	
	var confirmed_character = client_data[authority].get(ClientProperties.character, "")
	if !confirmed_character.is_empty():
		characters[confirmed_character] = authority
	
	data_updated.emit(authority, data)
	if !confirmed_character.is_empty() and confirmed_character != old_character:
		character_selected.emit(authority, new_character)
	
	sync_client_data.rpc(authority, data, characters)

func is_character_available(character:String):
	return !character.is_empty() and !(character in characters)

func get_data(authority:int) -> Dictionary:
	return client_data.get(authority, {})

func get_local_data() -> Dictionary:
	if !multiplayer.multiplayer_peer:
		return {}
	return client_data.get(multiplayer.get_unique_id(), {})
