# Schema

The editor can export conversations in json. Conversations are lists of objects
with the following formats:

# Hub
A position in the conversation.
 - conversation_id: a number uniquely identifying this hub
 - speaker: the speaker who "says" this hub. Default ""
 - text: the text content of this hub. Default ""
 - tags: tags that can be used to search for this hub. Separated by commas or whitespace. Default ""
 - choices: number of choices going out from this hub. Default 0
 - expressions: newline-separated code snippets that can be used to check against the story state before visiting this hub, or check against the story state after visiting this hub for the first time. Default ""
 - callbacks: newline-separated command-line style callbacks to the engine to do arbitrary actions after visiting this hub for the first time. Default ""
 - flags: integer bitflags of this hub. Affects dialogue flow and can be used to search for hubs by flags. Default 0
 - recipients: newline separated speakers. If not empty, overrides dialogue rules about privacy and displays this hub to only these speakers. Default ""
 - channel: if not empty, name of the channel this hub changes to. Default ""
 - priority: if not 0, the story priority of this hub compared to others in the channel. Default 0

# Connection
A connection between two hubs.
 - from_id: id of the hub this connection originates from
 - from_port: port on the hub this connection originates from
 - to_id: the id of the hub this connection connects to

# Speaker
 - name: the unique name of this speaker.
 - flags: flags of this speaker. Affects how players using this speaker can interact with a conversation. Default 0.
 - listening: newline separated list of speakers players using this speaker are considered as "listening to", in addition to the speakers they are already listening to.

# Camera
 - position:x: x position of the camera in the editor when last saved
 - position:y: y position of the camera in the editor when last saved
 - zoom: zoom in the editor when last saved

# Other notes
All fields may be empty to indicate default values.

Additional properties are also allowed in the schema and will be read by the engine as "user defined properties" or "additional data". Currently, the editor does not have strong plans to support this but the architecture supports quick experimental changes to the schema, and this additional data system ensures the engine and users of the engine have access to new/renamed fields immediately.