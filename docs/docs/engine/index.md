# Engine

The Louder engine is an open-source program meant to interpret conversations
created by the [editor](../editor/). The engine can manage, save, and load state
for conversations, manage the flow of conversations, and save and load replays
and checkpoints of a conversation. For details of how this works, see the [api docs](../rustdoc/louder_rust/)

This is very much a back-end tool, and does not help with rendering text, sound,
character portraits, etc. This deficiency is the purpose of the [examples](../examples/)
secion. Also, unless you are writing your game in Rust, you will probably be using
one of the [adapters](./adapters)

## Text and encodings
The engine never displays anything, so it has no opinion about what a byte
means. Strings cross its boundary as bytes and come back as the same bytes: a
host that hands over text in some encoding reads back that same encoding, and is
responsible for both halves of that. The one place the engine has to know is a
JSON document, since JSON is text and cannot be parsed otherwise. The encoding must be passed along with the json string in this case.

# Goals of Dialogue Rules
These three core values are what all structures and rules below exist to serve.
 - The engine should store and drive dialogue trees.
 - The engine should manage multiple users and dialogue trees easily.
 - The engine should be extensible in two aspects. First, it should be easy to extend single user trees to multi user trees using the structures provided. For example, introducing a new playable character or making an npc into a playable character should both be as simple as adding choices to a tree. Second, it should be simple to hook custom scripting behavior into the engine without compromising its existing structure.

# Structures
Structures are the objects managed by the engine.

"Names" are integers mapped to strings provided by the user. When the engine is just as a core library, names must be declared explicitly, but when the engine is wrapped in a framework that takes care of encoding, the user should be able to pass strings wherever a name can be passed. The context is responsible for avoiding name collisions. The "NONE" name is -1, and the "ZERO" name is 0. Both have no string mapping and are used as special codes.

"Sets" should be represented as lists of objects. And the engine should generally be tolerant of duplicate items in these lists.

"Positions" are integer ids that refer to nodes in a conversation (dialogue tree), paired with a channel, a name describing which "flow" this position belongs to. The default channel is ZERO. The id part of the position is unique within a conversation, and considered equivalent to POSITION_NONE -1 if it is invalid.

"Variants" are values that may be integers, floats, or names.

"Additional Data" are maps between names and Variants. 

"Flags" are 32 bit integers interpreted bitwise with named booleans for some or all 32 bits.

"Unrecognized Objects" objects that are not recognized by the engine may be produced by the editor. A conversation created by the editor may have hubs, voices, and other objects that are discarded when the stream is loaded.

"Optionals" are objects which may or may not be present.

"Unions" are special variants with different possible types depending on context.

## Context
A context is a set of actors, voices (previously called speakers), conversations, events, and interrupts.

## Actors
An actor represents a user and is composed of an integer uniquely identifying this actor within the context, 2 disjoint sets of voices, one which they "speak as" and another which they "listen to", flags, a position, and additional data.

## Voices
A voice represents a voice which a user can use to interact with a conversation. Voices are composed of a name uniquely identifying this voice within the context, a set of voices which they "listen to", flags, and additional data.

## Conversations
The conversation represents a dialogue tree and is composed of a name uniquely identifying this conversation within the context, a set of hubs and a set of connections

## Hubs
A hub represents a position in a conversation and is composed of the integer id of a position (unique within the set of hubs of a conversation), optional channel, an integer "priority", optional voice, a set of voices called "recipients", flags, a name "text", a number of choices, a set of names "expressions", a set of names "tags", and additonal data.

## Connections
Connections are composed of two integer ids of positions in a conversation "from" and "to", as well as an integer "port" into the number or choices available at the "from" hub.

## Choices
In the presence of an Actor, the word "choice" is used to describe a hub for which there exists a connection from the actor's position to that hub.

## Events
Events are unions of Motion events, Actor added events, Actor removed events, Voice added events, Voice removed events, Conversation added events, and Conversation removed events.

### Motion events
Motion events are composed of a start position, end position, and actor.


### Added/removed events
Added/removed events of type X are composed of X's unique identifier (integer for Actor, name for Voice and and Conversation), as well as a copy of the object itself.

## Interrupts
Interrupts are composed of an actor and the name of an expression at the hub of the actor's current position.

# Rules
Rules are named interactions for the structures described in Structures. As per the first goal, the primary goal of the engine is to provide an action of an actor making a choice. If multiple rules say "an actor can make a choice if", they will exhibit OR logic on a choice being available. If rules say "an actor cannot make a choice if", they will exhibit NAND logic on a choice being available. 

Conversations are made up of hubs connected by choices. 

Conversations are interfaced with via Conversation Actors, entities which can speak as
and listen to different speakers.

As actors make dialogue choices, hubs "appear" to them.

To see how the objects of dialogue are encoded by the editor, see [schema](/editor/schema.md)

# Hubs
Hubs have a position, tags, speakers,
text content, choices, expressions, callbacks, flags, recipients, channels, priority, and user-defined properties.

## Position
A number uniquely identifying each hub in a conversation.

## Tags
Tags are lists of short strings attached to each hub that can be used to query the conversation as a whole.

Good for finding the "start" node, or filtering for start candidates and picking a specific starting point based on game state.

## Speakers
Speakers are single names on each hub. If present, they determine who the hub is visible to.

Speakers are themselves defined by their own name, flags, "listening" and additional data:
 - Name: The name used to reference this speaker in hubs and by actors.
 - Flags: Speakers can be flagged "public". If a speaker is not defined,
 they are assumed to be public. 
   - Public speakers. Hubs with public speakers appear to all conversation actors. Non-public speakers 
   only appear to actors who are explicitly "listening" to them. See section on "Listening"
 - Listening: Speakers can reference other speakers as "listening." If an actor is speaking
 as a speaker, they implicitly listen to these speakers as well.
 - see Homogeneous Objects.

When this guide says that a hub "appears" to a speaker, it's referring to
appearing to an actor that is either speaking as or listening to this speakers.

## Text Content
Arbitrary text content of the hub. Typically this is what you would want to
display as the content of your dialogue, along with the Speaker.

## Choices
Choices have some text content describing how the choice should appear when this
hub appears. Each choice references another hub by its position.

Actors traverse through dialogue by "making choices" or "selecting choices",
which changes their position and all relevant information to match the linked
hub under an available choice.

Choices should only appear to an actor if their linked hub would appear, and the
speaker of the linked hub matches one of the four cases:
1. The speaker is in the actor's "speak as" list
2. The speaker is in the actor's "listen to" list and no other actor has that speaker in it's "speak as" list.
3. The speaker is public and no other actor has that speaker in it's "speak as" list.
4. The speaker is in the actor's "speak as" list, "listen to" list, or is public, and that choice is the only choice on this hub. This is called the "spoke rule" and it prevents linear dialogue from switching "leading" actors too frequently. 

If the hub has "recipients", these checks are done on the recipient speakers instead of the hub's speaker.

## Expressions
Snippets of code sent to the engine's expression parser. Can directly manipulate
story state.

Expressions are either classified as "conditions"- those which check story state,
or Statements- those which modify story state.

Conditions are checked every time a hub is checked for whether it should appear,
if any condition on a hub evaluates to 0, false, or an empty string (see Variants),
the hub will not appear.

Statements will only run if the hub has not been 
traveled to before. The data of which hubs have been "covered" is stored in the
story state along with variables.

## Callbacks
can be caught as events when this hub is traveled to.
Space-separated commands that are exposed to the program using the engine and

Callbacks are assumed to be mutable and only run once unless flagged as "repeating."

## Asks
Expressions can contain callbacks using the syntax `$(<callback>)`. For example:
`$(get hero money)`. These callbacks are called "asks." The engine provides an
interface for getting queued asks and replying to them. When evaluating an
expression, the engine will _wait_ until all contained asks are replied to.

Asks happen as a result of choices in dialogue, like other events. However,
the engine will keep track of which tasks are being held up by which ask routines.
When matching verdicts are returned, the task is completed, emitting regular
events and completing the dialogue action.

This is good for complex logic or if the user does not want to use variable
naming rules. See Story State for variable naming rules.

## Flags
Checkboxes that hubs can have checked:
 - Auto. Auto hubs automatically select the first choice available from the
 perspective of the actor that entered them. Auto nodes never appear. If no
 choices are available, this is considered an error. 

Hubs can also be implicitly auto under very specific conditions. This is called
the "singleplayer rule". If hub A was reached from a hub B that had multiple
choices, and A is only visible to the actor that made the choice, it should
behave as an auto node temporariliy with all the implications that follow from
that. In this case, hub A behaves like a "player choice" node in a more traditional
singleplayer system, which the user may want to have progress automatically.

## Recipients
List of names, like tags. If not empty, the hub may only appear to actors
speaking as or listening to speakers in this list.

## Channels and Priority
Channels and priority allow for multiple dialogue flows to be balanced at once.

Channels are strings, like tags, and work like they would in a normal messaging service. If a hub has a channel, it changes
the channel an actor is "in" when it appears to them, and the actor stays in that channel until another
hub changes it again.

By default, the channel is "main."

Actors in different channels do not see hubs that appear from
choices made by actors in other channels. Thinking in terms of the Private flag, 
channels act like a "privacy group" around the actors inside them.

Priority is a number, and takes care of what happens when multiple actors
are in the same channel, but different parts of the conversation. 

By default, the priority of every hub is 0. "The priority of the hub at an actor's position" below is shortened to "the priority of the actor."

When an actor, the "leader", makes a choice, and there are other actors who 1. are in the same channel as the leader before or after their choice, 2. are actors who the hub at the new position of the leader may appear to, and 3. have a priority less than or equal to the leader's after their choice, these actors "warp" to the leader.

When an actor makes a choice, and there are other actors in the same channel to whom the new hub may appear, but won't follow the leading actor because of priority rules, the hub will appear to these actors in the "background" (see Events and Interface), without choices. Again, thinking of a messaging service, this is like separating an actor's "thread" from the rest of the channel- still visible to them.

Finally, when a choice is made, the resulting hub appears to all spectator actors regardless of their channel, and do not appear in the background. Spectators are treated as not having a channel or priority.

## Warping
When one actor, the "source," warps to another, the "target," they will change their position to match theirs,
and change all information they have that depends on the hub at their position
to match the hub under the target.

If the warping is happening due to the target leaving an Auto hub, the source is warped instead to the result of the source actor making passing through the same Auto hub. This special "emulation" warp choice can be made regardless of whether the Auto hub should be available to the source actor. This is the main mechanism by which splitting happens between actors, since the replayed choice can end up going down different paths through Auto logic.

An emulating actor is not deciding anything, so the first branch available to
it may be none: every branch may be a voice someone else is playing, which is
exactly the case where the Auto hub is meant to send the two actors apart. The
source then takes the first branch that would *appear* to it, and only if no
branch appears either does it end up where the target did. The source is
therefore gated on its own landing rather than the target's, since the two are
the whole point of the split.

## Changing position
Simply changing the position of an actor is relatively simple. If the actor lands
in a hub flagged as Auto, the logic of Auto nodes will proceed as normal and the 
actor will land somewhere else. If the hub at the new position may not appear to
the actor, this is treated as an error.

# Actors
Actors have a list of speakers they "speak as", "listen to", flags, current
position, current channel and priority, and additional data.

## Speak as and Listening
See Speakers, and then Choices to see how these settings interact with the engine.

## Flags
 - Spectate. All non-auto hubs appear to this actor, choices do not appear to
 this actor unless Debug is flagged.
 - Debug. Auto hubs and all choices appear to this actor.
Debug and Spectate actors cannot belong to channels.

## Current channel
The channel of the last hub that appeared to this actor with a channel. See Channels and Priority.

## Additional data
Additional data behaves like channels. When a hub appears to an actor with
additional data, that additional data changes the actor's additional data. The
actor then keeps this data until another hub changes it.

# Homogeneous Objects
A homogeneous object that contains a mapping between string names and Variants.

The "additional properties" of hubs are homogeneous objects.

# Story State
The story state is the memory where variables are mapped to values. It is
composed of homogeneous objects.

Variable names starting with `/_+[A-Z]/` (`_A` or `__Shared`) are shared across conversations and not saved.

Variable names starting with `/_+[a-z0-9]?/` (`_a` or `__1`) are local to each conversation and
not saved.

Variable names starting with `/[A-Z]/` are shared and saved.

Variable names starting with `/[a-z0-9]?/` are local and saved.

The user can also specify their own naming rules, like variables "Inventory_..."
can be items in the player's inventory. However, naming rules can cause conflicts
and become hard to manage and synchronize. If this becomes the case, use Asks
instead.

## Variants
Variants are integers, floats, booleans, or strings.

In expressions, variants can be operated on with the standard array of
arithmetic and logic operators, plus concatenation logic for strings.

Variants can always be treated as integers. Floats truncate, strings go to zero,
and booleans become 1 or 0 for true and false respectively.

# Motion
A source position, target position, an actor, and parent conversation associated with a choice selected. In the events
section below, the "associated motion" is the action on the conversation that
caused the event to be emitted.

# Events and Interface
Events are emitted when:
 - A hub is revealed to an actor. Provides identification for the actor, whether the hub is in the "background" or not, associated motion
 - A choice is revealed to an actor. Provides identification for the actor, choice text, choice index, linked position, associated motion
 - A callback should be executed. Provides the callback to execute, associated motion
 - The story state changed. Provides the source statement (if available), changed variable, new value and associated motion.
 - A condition evaluates. Provides the condition evaluated, resulting value, and associated motion.
 - A warp happens. Provides whether it was an "emulation" warp or not, the channel, the source actor, the resulting position, and associated motion.
 - An actor's channel changes. Provides the actor, channel and associated motion.
 - An actor's priority changes. Provides the actor, channel and associated motion.
 - An actor's additonal data changes. Provides the actor, additional data and associated motion.
  - An actor is added. Provides the actor, speak as and listen to speakers.
  - An actor is removed. Provides the actor.

One event is not about a conversation running: a record of a loaded document that
the engine could not read is reported as a load error event, carrying the
position of the record at fault, or -1 when the document as a whole was the
problem. A load reports these through events rather than failing outright, so one
malformed record does not cost the caller the rest of the conversation.


Finally, the engine exposes an interface to see the following:

Constant accessors. Channels are optional and default to "main".
 - From an actor and conversation, get the current channel and position
 - From a actor and conversation, get choices currently appearing to this actor
 - From a channel, get current actors in channel
 - From a channel and position, get current actors in channel at that position
 - From a hub position and conversation, get any hub property, including object properties
 - From an actor and conversation, get any actor property including additional properties
 - From a speaker, get any speaker property including additional properties
 - From a conversation, get any Variant from the story state
 - From tags, get hub positions
 - From flags, get hub positions
 - From channels, get hub positions
 - From priority constraints, get hub positions
 - From a speaker, get actors speaking as that speaker
 - From a speaker, get actors listening to that speaker
 - Get spectators
 - Get debug actors

Getters that return multiple of something should generally be in the form of
an iterator, taking a blank object as the starting argument and returning a
blank object when all objects have been iterated through. Since objects in the
conversation, actors, and so on are generally registered via integers, there is
always a canonical ordering with which to iterate through these sets without
having to expose complex collections to the user.

Modifiers
 - Load a conversation from a file created from the editor
 - Unload a conversation
 - Load actors and speakers from a speaker config created from the editor and add them to a conversation
 - Add an actor with speak as, listen to config to a conversation
 - Add a spectator to a conversation
 - Add a debug actor to a conversation
 - Remove an debug, spectator, or regular actor.
 - Select a choice from an actor, choice index and conversation
 - Force a warp from an actor, target, optional target choice index, and conversation
 - Force a change of position from an actor, position, and conversation.
 - Force a change of position from a channel, position, and conversation.
 - Force a channel change with an actor, channel, and conversation
 - Change the story state with a variable, variant and conversation

All functions that take a conversation, actor, or position should return errors
for when any of those objects are invalid. Story state variables, however, 
cannot be invalid, and default to 0 if they haven't been initialized.

# Configuration
The spoke rule and singleplayer rule can be configured by making calls to the
engine.

# Example Flows
!!!WARNING Extremely boring technical program flow ahead! Not really practical? More to help me implement and test the code. Lol.

User loads a conversation with hubs 0, 1, 2, 3, 4. 

```
Speakers:
2: "Speaker A"
3: "Speaker B"

Choices:
0: "A Choice to Make" -> 1
1: "(invisible choice 1)" -> 2, "(invisible choice 2)" -> 3
2: "Finish Conversation" -> 4

Flags:
1: Auto

Channels:
2: "Channel A"

Tags:
0: "Start"

Callbacks:
4: "exit"
```

User loads a config with speakers "Speaker A" and "Speaker B". (under the hood,
these may still be identified with integers like everything else, I'm not sure yet)

```
Names:
"Speaker A": "Speaker A"
"Speaker B": "Speaker B"
```

Actors 0 and 1 are then added automatically:
0: Speaking as "Speaker A"
1: Speaking as "Speaker B"
 - Two "actor added" events are emitted matching the data.

The user catches these events and shows a debug message.

The user then adds a spectator + debug actor, 2.
 - One "actor added" event is emitted with no speak as / listen to data, but with the spectate flag set.

The user searches for nodes tagged "Start" and forces the "main" channel into position.
 - Three hub appearance events happen, one for actor 0 and another for actor 1, and one for the debug actor 2
 - Three choice appearance events happen for the same actors, both for the single "Choice index 0" from hub 0 for each actor.

The user listens for these events. First, they report the debug information to the terminal:
 - `Actor 0 is at position 0 with choices "Choice index 0". (channel: "main")`
 - `Actor 1 is at position 0 with choices "Choice index 0". (channel: "main")`
The user then repeats them over a network to be seen by
clients, along with packets telling them to only accept appearance events for
actors 0 and 1 respectively. The client programs get their respective events and
display the hub and choice.

Client 0 sends the choice at index 0 (conveniently labeled "Choice index 0") to
the server and the user selects the choice.
 - One hub appearance event happens immediately for actor 2: the Auto node 1. This is ignored by the other actors.

Auto logic happens: The Auto node 1 looks at its choice index 0 from the perspective of actor 0 and checks if actor 0 can go down this path.
"(invisible choice 1)" may appear to actor 0 because:
 1. Actor 0 speaks as "Speaker A".
 2. "(invisible choice 1)" links to hub 2.
 3. Hub 2 has speaker "Speaker A".
 4. Hub 2 has no failing conditions.
 5. Hub 2 is not flagged Private with a recipient property other than "Speaker A" (or if Actor 0 had other speakers it spoke as, say "Speaker C", Hub 2 could still be flagged Private with recipient "Speaker C" and be visible here.)

Therefore:
 - Two hub appearance events happen for actors 0 and 2, updating the network client and the debug terminal.
 - One channel change event happens 
 - Two choice appearance event happen for actors 0 and 2, updating the network client and the debug terminal.

Now, the warping logic happens. 
 1. Because Actor 0 started in channel "main", the same channel as actor 1, actor 1 warps to actor 0. 
 2. Because Actor 0 moved into an Auto hub, this is an emulation warp of actor 1 through hub 1.

Auto logic happens again, but this time the first choice does not pass.
"(invisible choice 1)" may *not* appear to actor 1 because:
 1. Actor 1 does not speak as "Speaker A"
 2. If this choice were allowed, there would be 2 choices available.
Therefore, the choice cannot appear to actor 1, so auto logic skips it. The second choice succeeds though:
 1. Actor 1 speaks as "Speaker B"
 2. No failing conditions, etc. etc.

Therefore:
 - Two hub appearance events happen for actors 1 and 2, updating the other client and terminal.

No choice event is fired because there are no choices from actor 1's new position.

In summary: Actor 0 is in position 2 with one choice in a newly created channel. Actor 1 is in position 3 with zero choices.

The network requests actor 0 to make their choice. The user passes this request
to the engine. Actor 0 moves to hub 4.
 - Two hub appearance events for actors 0 and 2. Even though hub 4 may appear to actor 1, 1 is in a different channel.
 - One callback event is sent with the callback "exit". The user interprets this as the end of execution.

Here are three equivalent conversations achieved with priority and private flags instead of channels:

## With Priority
By replacing the channel on node 2 with a priority on node 4, node 4 is no longer high enough priority to drag actor 0 along with actor 1's choice.
```
Hubs 0, 1, 2, 3, 4. 

Speakers:
2: "Speaker A"
3: "Speaker B"

Choices:
0: "Choice index 0" -> 1
1: "(invisible choice 1)" -> 2, "(invisible choice 2)" -> 3
2: "Finish Conversation" -> 4

Flags:
1: Auto

Priority:
4: -1

Tags:
0: "Start"

Callbacks:
4: "exit"
```

Or, equivalently, give the lower branch of the auto node higher priority. Since that node cannot appear to actor 0, it can't follow.
```
Hubs 0, 1, 2, 3, 4. 

Speakers:
2: "Speaker A"
3: "Speaker B"

Choices:
0: "Choice index 0" -> 1
1: "(invisible choice 1)" -> 2, "(invisible choice 2)" -> 3
2: "Finish Conversation" -> 4

Flags:
1: Auto

Priority:
3: 1

Tags:
0: "Start"

Callbacks:
4: "exit"
```

## With Privacy
```
Hubs 0, 1, 2, 3, 4. 

Speakers:
2: "Speaker A"
3: "Speaker B"

Choices:
0: "Choice index 0" -> 1
1: "(invisible choice 1)" -> 2, "(invisible choice 2)" -> 3
2: "Finish Conversation" -> 4

Flags:
1: Auto

Recipients:
4: "Speaker A"

Tags:
0: "Start"

Callbacks:
4: "exit"
```

## Experiment
What happens if we move the "Channel A" property in the original conversation from node 2 to 4?
```
Speakers:
2: "Speaker A"
3: "Speaker B"

Choices:
0: "Choice index 0" -> 1
1: "(invisible choice 1)" -> 2, "(invisible choice 2)" -> 3
2: "Finish Conversation" -> 4

Flags:
1: Auto

Channels:
#2: "Channel A" get rid of this
4: "Channel A"

Tags:
0: "Start"

Callbacks:
4: "exit"
```

Node 4 doesn't have a speaker. A speaker that is not defined is public by
default, so it may appear to both actors 0 and 1. When actor 0 moves there,
it's moving from node 2, channel "main", into node 4, priority 0. Actor 1 will also be in channel "main" with priority 0. This matches the conditions for a warp. 

Since actor 0 is making a choice, this warp acts as if actor 1 were at node 2, making the same choice as actor 0. Since there is no auto logic, actor 1 goes the same way as actor 0, and both end up in node 4.

This still results in a single callback event, since callbacks only execute once on each hub before they are marked as "covered."