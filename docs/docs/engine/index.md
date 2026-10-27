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

To see how the objects of dialogue are encoded by the editor, see [schema](/editor/schema.md)

## Context
A context is a set of actors, voices (previously called speakers), conversations, events, interrupts, and a config.

## Actors
An actor represents a user and is composed of an integer uniquely identifying this actor within a conversation, the name of their parent conversation, 2 disjoint sets of voices, one which they "speak as" and another which they "listen to", flags, an set of positions called their "history", with the last position just being called their "position", and additional data.

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

For example, if connections from 0 to 10, 0 to 20, 0 to 30 exist, then "choice 1" for an actor at position 0 describes hub 20.

## Events
Events are unions of Motion events, Actor added events, Actor removed events, Voice added events, Voice removed events, Conversation added events, and Conversation removed events.

### Motion events
Motion events are composed of a start position, end position, and actor.

### Added/removed events for Actors
Actors emit events when they are added/removed *to conversations*, not to the context. Event is comprised of actor id, conversation name, Position where actor was added/removed, and a copy of the actor object.

### Added/removed events for Voices and Conversations
Added/removed events of type X are composed of X's name, as well as a copy of the object itself.

## Interrupts
Interrupts are composed of an actor and the name of an expression at the hub of the actor's current position.

## Configs
The config adjusts the rules of the engine.

# Rules
Rules are named interactions for the structures described in Structures. As per the first goal, the primary goal of the engine is to provide an action of an actor making a choice. Making a choice describes updating an actor's position using one of its choices. If multiple rules say "an actor can make a choice if", they will exhibit OR logic on a choice being available. If rules say "an actor cannot make a choice if", they will exhibit NAND logic on a choice being available. For example, actors can make a choice if rules A or B or C apply, and not if D or E or F apply, so if rules A B D apply for a particular choice, that choice is not available with reason D. 

When asked what choices are available for a given actor (choices(x, A) for choices at hub x or choices(A)), the engine should return a list of booleans and Unions, one for each choice at the actor's current position. If the choice is available, the first rule that made it available is the value of the union. if the choice is unavailable, the first rule that made it unavailable is the value of the union. the boolean stores availability. The philosophy here is transparency, the user always knows why a choice is available or not.

The same is true for other interactions introduced in the rules section. For example, with visibility, "visible to actor" rules would exhibit OR behavior and "not visible to actor" rules would exhibit NAND behavior.

This is not a spec for the code syntax of the engine. This is more of a mathematical formal syntax to strongly describe behaviors so engine implementation is more straightforward.

Rules assume the presence of a context and a conversation.

Rules are ordered. If rule R was stated *before* rule S below, and there is a contradiction, rule R should be followed with a warning in the implementation that they conflict.

Rules are exposed to the user as an enum, alongside a function that converts from rules to rule names.

In implementation, this probably won't be realized as a big global list of rules, but this list should remove any ambiguity on the correct behavior for any given scenario.

## Meta Rules
Rules about rules.

### Copy Rule
A rule R can be copied as a new rule S. All behaviors are copied under a new name, then other properties are overridden as described.

### Experiment Rule
All rules can be enabled/disabled via a flag in the config.

## Visibility Rules
Determines whether hubs should appear to actors.

### Public Voice
Voice v can be flagged as PUBLIC. In this case, flag(v) | PUBLIC

### Private Hub
Hub x can be flagged as PRIVATE. In this case, flag(x) | PRIVATE

### Visible Hub
Hub x can be "visible" to actor A. In this case, visible(x, A)

### Priority
Hub x has priority(x). An actor A positioned at hub x has priority(A) = priority(x)

### Hub Voice
voice(x, A) = voice(x) if voice(x) is present, the first element of speak_as(A) if speak_as(A) is not empty, and an empty optional otherwise.

### Visible Speaker
Voice v can be "visible" to actor A. In this case, visible(v, A)

### Visibility rule 1
if voice(x, A) in speak_as(A), visible(x, A)

### Visibility rule 2
if voice(x, A) in listen_to(A), visible(x, A).

### Visibility rule 3
if flag(voice(x, A)) | PUBLIC, visible(x, A) for all actors A.

### Visibility counter 1
if flag(x) | PRIVATE and recipients(x) \cap speak_as(A) is empty, !visible(x, A)

### Spectator Rule
An actor A can be a "spectator". In this case, tag(A) | SPECTATOR.

### Spectator Visibility
If tag(A) | SPECTATOR, visible(x, A)

This rule is stronger than visibility counters. Even if another rule says !visible(x, A), tag(A) | SPECTATOR overrides that decision.

## Choice Rules
Rules that determine whether choices are or are not available.

### Choice Hub
choice(x, A) is defined if there exists a connection c from position_id(A) to x, and may be "available", but by default is unavailable

### Choice Text
If choice(x, A) is defined, choice_text(x, A) returns text(x)

### Standard Rule
if voice(x, A) in speak_as(x, A), then choice(x, A) = available, standard

### Skill Rule
if voice(x, A) in listen_to(x, A) and voice(x, A) not in speak_as(x, B) for all B != A, choice(x, B) = available, skill

### Visibility Check
if !visible(x, A), choice(x, A) = unavailable, invisible

### Removal Rule
configurable with optional removal tag and optional removal text.

if choice(x, A) is not available for all choices x, choice(POSITION_NONE, A) = available, removal and has the effect of removing A from the conversation.

if removal tag is configured, rule only applies if the removal tag is in tag(x)

if removal text is configured, and choice(POSITION_NONE, A) = available, removal, choice_text(POSITION_NONE, A) is removal text

### Meaningful choice rule
if choices(x, A) had more than 1 element, meaningful(x, A). if choices(A) has more than one element, meaningful(A). Otherwise, !meaningful(A)

### Softlock Rule
Copy of removal rule. If the rule applies, choice(POSITION_NONE, A) is undefined and choice(x, A) is defined if and only if x is the most recent hub in A's history where meaningful(x, A)

Conversations are made up of hubs connected by choices. 

Conversations are interfaced with via Conversation Actors, entities which can speak as
and listen to different speakers.

As actors make dialogue choices, hubs "appear" to them.

### Spoke Rule
If choice(x, A) is defined, visible(x, A), choice(x, B) and !meaningful(B) for all B != A, choice(x, A) = available, spoke. 

Basically, if A is in a spot where other actors can move, but don't have meaningful choices, and A could move in their place without breaking visibility rules, A can.\

### Debug Actors
Actors can be "debug actors", in this case, tag(A) | DEBUG

### Debug Rule
if choice(x, A) and tag(A) | DEBUG, choice(x, A) = available, debug

This rule is more powerful than rules that would make choice(x, A) unavailable.

## Control Rules
Rules for controlling dialogue

### Auto Hubs
Hub x can be flagged AUTO. In this case, tag(x) | AUTO. 

### Auto Traces
Let y be the first element of choices(x, A), ordered by port of the associated connection.

if not tag(x) | AUTO, auto(x, A) = x

If tag(x) | AUTO, and not tag(y) | AUTO, auto(x, A) = y.

If tag(y) | AUTO, y = the first element of choices(y, A). Repeat until not tag(y) | AUTO or choices(y, A) is empty. If choices(y, A) becomes empty, auto(x) is undefined. Otherwise, auto(x, A) = y after repetition.

### Auto Trap Rule
If choice(x, a) and auto(x, A) is undefined, choice(x, A) = unavailable, auto_trap

### Standard Choose
If choice(x, A) is available, choose(x, A) = standard, x, moving A to x and returning the rule which caused it. The expression "If choose(x, A)" means "if actor A makes choice x".

### Auto Choose
if choose(x, A) and auto(x), choose(x, a) = auto, auto(x).

### Auto Visible Rule
If tag(x) | AUTO, visible(x, A) for all A.

### Singleplayer Rule 1
If choose(x, A) != singleplayer, visible(x, A), and !visible(x, B) for all B != A, choose(y, A) = singleplayer

The "!= singleplayer" clause prevents recursion of this rule.

### Singleplayer Rule 2
If choose(x, A), !visible(x, A), and !visible(x, B) for all B != A, choose(y, A) = singleplayer_2

This is singleplayer rule 1, but with recursion allowed for the special case where there are nodes nobody can see.

### Warp Rule
If choose(x, A), visible(x, B) and priority(x) >= priority(B), choose(x, B) = warp

Here we have an emergent behavior with Auto Rule and Auto Visible Rule. if tag(x) | AUTO, then visible(x, B) and choose(x, B) = warp, then choose(y, B) = auto

### Non-warp Rule
If choose(x, A), visible(x, B) and priority(x) < priority(B), do nothing

This rule is named for the purpose of event calling via other rules.

### Force Position Rule
if force(x, A), choose(x, A) = force

### Evaluation Rule
evaluate(x, A) is the result of evaluating all expressions on x from the perspective of A, a boolean paired with a rule. evaluate(x, A, i) is the result of evaluating the i'th expression, starting from 0, and is a Variant paired with a rule.

See Expressions for more about how evaluation works.

### Standard Evaluation
If choose(x, A), evaluate(y, A) for all y in choices(x, A)

### Evaluation Andedness
If evaluate(x, A, i) is not 0 and evaluate(x, A, i) is not an empty name and evaluate(x, A, i) is not NONE for all i, evaluate(x, A)

### Evaluation visibility counter
if !evaluate(x, A), !visible(x, A)

This is a control rule and not a visibility rule because evaluate(x, A) describes an action, which can have side effects, rather than a getter.

A second crucial emergent behavior, since choice(x, A) is unavailable if invisible, evaluating an expression can hide a choice.

### Interrupt Rule
evaluate(x, A, i) can equal _, interrupt (see Expressions)

### Parse Error Rule
evaluate(x, A, i) can equal 1, parse_error
(see Expressions)

### Link Hubs
A hub x can be tagged as a link hub. In this case, tag(x) | LINK

link(x) is an id for another hub.

### Default Rule
Every property of a hub x, or voice v has a default value defined in the (schema)[../editor/schema.md].

### Domination Rule
For two hubs x, y, x * y is a hub in the position of y, however, every property
for which x has a non-default value replaces the same property of y.

For example, if tag(x) | PRIVATE, and recipients(x), tag(x * y) | PRIVATE and recipients(x * y) = recipients(x)

### Link Rule
If tag(x) | LINK, x = x * y.

## Event Rules
Rules for observing dialogue

### Appearance Rule
appear(x, A) is an event where hub x appears to actor B. A hub "appearing" contains information about the hub, and the choices that should be available to B from that hub. results of this function are of the form appear(x, A) = e, rule (though the user should generally be able to access the rule that caused an event from the event object itself)

### Standard Appearance Rule
If choose(x, A) = rule, appear(x, A) = e, rule, where e is the event of x appearing to A.

### Background Rule
if B has position y != x, appear(x, A), and priority(y) > priority(x), appear(x, B) = e, background

### Ask Rule
ask(y, A, i) is an event that asks the user to resolve the i'th expression on y from A's perspective 

if evaluate(y, A, i) = n, interrupt, n is the name of the expression that is being deferred to the user. hang and ask(n) = e, ask

"hanging" means not exiting the execution stack. If evaluate(y, A, i) is being called because of evaluate(y, A) (Evaluation Andedness), the program hangs by not exiting from this call stack. The opposite of hang is resume.

### Answer Rule
answer(n, r) is a user-made event to unhang the program.

if hang, ask(n, i), and answer(n, r), then evaluate(y, A, i) = answer(n, r) = r, answer and resume 

### Warp Event Rule
warp(x, A) is an event indicating A has (or has not) warped to x. results are of the form e, rule

### Do Nothing Rule
if a rule said "do nothing" about a warp, warp(x, A) = e, warp_failed

### Do Something Rule
if a rule said choose(x, A) = something about warping, warp(x, A) = e, warp_success

## Channel Rules
Rules about channels

### Channel Rule
channel(A) is the name of the current channel of A. ZERO by default

channel(x) is the name of the channel set by x, if any. NONE by default

### Channel Change
if choose(x, A), channel(x) != NONE and channel(A) != channel(X), channel(A) = channel(x)

### Channel Change Warp
if B was at position x and channel(B) = channel(A), choose(y, B), and visible(y, A), channel(A) = channel(B) and choose(y, A) = channel_warp

### Channel Separation
if B was at position x and channel(B) != channel(A), choose(y, B) and channel(B) != channel(A) still, do nothing

# Expressions
By default, expressions are parsed using an internal parser with the following syntax:

Expressions are made up of identifiers, literals and operators. Each identifier represents a function.

Operators are the same as C, without pointers, offsets, or precompiler declarations, with the same precedence rules.

Identifiers are any string beginning with an alphabetical character or an underscore, and consisting entirely of alphanumeric characters and underscores. 

There are also special identifiers, which begin with $.

Finally, there are literals for integers, floats, and strings.

Whitespace is used to separate symbols. Any character not caught by the above categories, such as $ in the middle of an identifier or #, is treated as invalid.

The expression `identifier` asks the user for the value of the function "identifier" with no arguments, using an interrupt. Then, `identifier` is replaced with the literal for that value.

An expression made up of literals and operators alone has a known value.

The expression `a(b, c)` first evaluates b and c. If b is only made up of literals and operators, the value is known already. If not, the user is asked the value of b. Same with c. Once b, c are resolved, the user is asked the value of a, which must be an identifier in this expression, with arguments equal to the literals that b and c resolved to. If a is not an identifier, the expression is invalid. 

Again, we have to make an exception to the aspiration about encoding-free implementation, since the encoding of expressions must be known to parse them. The encoding used by expressions can be set in the context config.

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
