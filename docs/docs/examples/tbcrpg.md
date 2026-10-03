# Simplified CRPG
The purpose of this example is to provide the simplest possible demo which
takes advantage of all the engine's features:
 - Linking to other programming languages
 - Multiplayer conversations
 - Multiple conversation loading
 - Complex branching conversations

RPG will be mostly text based with some images displaying over the main text-based display.

## Main Menu
The demo starts with a menu that describes the area where the player is standing
and lists interactable objects. There are three playable characters: Geno, Reka
and REDACTED.

"You are standing in The Silent Square... (additional description here)"
 1. Speak to bandit.
 2. Speak to Hagieph synod.
 3. Speak to board game hustler.
 4. Speak to merchant (visible to Reka and REDACTED- speaks to Geno)
 5. Speak to pilgrim (visible to Geno and REDACTED- speaks to Reka)
 6. Speak to fish-man (visible to Geno and Reka- speaks to REDACTED)
 7. Check inventory
 9. Check skills
 0. Change character

## Dialogue
Options 1-6 open a short conversation and sends the message "blank and blank start talking to each other" to all players. If Geno speaks to Reka, that will open
the same conversation as Reka speaking to Geno, though it may start at a
different point (located using the [tags](/engine##Tags) system). And for any player character that speaks to one of the 3 npcs,
the same is true. 

For the 2-player pc-pc conversations, other pcs can "spectate" but generally
don't have options.

Each conversation is also manually tagged with the names of characters which are
"active" participants. Other players that join will get a message "blank and blank are still talking" and can leave the conversation whenever they do
not have choices.

If 2+ player characters are speaking to an npc, and that conversation
concludes, the conversation will change to the conversation between whichever 2
pcs spoke to the npc first. Other pcs will get the message that "blank and blank
start talking to each other" and return to the menu.

If multiple channels are visible to a player at a given time, the player will see
two columns of dialogue. One where they are making choices, and another where
the "background" hub events are being recieved.

## Skills Menu
Reaching key points in conversations will give a player experience points, which
become Levels after certain thresholds (see Levels). When the player gains experience, they will
get a message saying "x points to next level up." or "Level point gained!". 

In the main menu, they will be shown their current total level and number of
unallocated level points. e.g. the total number of times they have passed a
level threshold the the total number of times they have passed a level threshold
without spending the point they got from that.

Selecting "Check Skills" will show a list of the character's skills, along with
the current level of each skill. When the player presses 1-6, the skill's 
description will display to the right of the list.

If level points are available, the player can press Enter after allocating a
skill to make an "Are you sure you want to spend a skill point to upgrade \[Skill Name]?".
If the player presses enter again, the the level point is spent and the skill
level is permanently upgraded.

Certain points in conversations will initiate a callback for a "dice check",
where a 2d6 will be rolled, and a skill's level added, then checked against a 
threshold before the result is sent back to the engine.

## Inventory Menu
Checking inventory will show a screen with the player's items. This demo should
only have so many items as can appear on screen at one time. Some items are
"Equipment." Equipment has a named Equipment slot:
 1. Head
 2. Neck
 3. Torso
 4. Belt
 5. Pants
 6. Shoes
 7. Jacket
 8. Left Hand
 9. Right Hand
And each character is wearing 0 or 1 items in each of these slots. Equipment has
an array of
additive bonuses/penalties to certain skills, each followed by 3-10 word explanations
of why that equipment piece has that effect.

Items display in a 3x6 grid and can be focused with Tab/Shift+Tab and Equipment can be equipped/swapped for
worn equipment using the Enter key. The focused item will have its description
displayed to the right of the menu. Equipment with skill bonuses/penalties will have its explanations
displayed along with its description.

Bonuses can be applied to a skill name or category. To the player, the bonus
will always appear to be for their skill either matching the skill name or in the appropriate category.
Items with bonuses applied to a specific skill name are intended to only be held/equipped by
the character which has that skill.

The player can also press the number keys to either remove equipment or, if the
focused item can fit in the same equipment slot, swap the current equipment in
that slot with the focused item.

Finally, the player can press the arrow keys to move items around in the grid,
swapping with whichever items are in the way.

## Additional notes on menu functionality
Checking Inventory and Checking Skills can be escaped using the Esc key. If a
player can leave a conversation, they will be presented the option to leave
as a choice bound to a number button, or they can leave by pressing Esc.

"Change Character" will return the player to character selection. The flow of
connecting to server -> selecting a character -> going to the menu is described
in more detail in the Architecture section.

# Characters
## Geno
Geno is a merchant selling a ceremonial dagger and a rusty sabre.

His goal is to get the most money, but he's also motivated to win against people
in games.

| Skill       | Category     | Level | Description                                                                                                                                                                                                                                                                                                                                                                     |
| ----------- | ------------ | ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Cage        | Strength     | 3     | When my persona is sufficiently challenged, I retreat into my cage. The cage protects me from everything, whether I want it to or not. I need my cage to do this or I will go insane. My life may never get better because of it, it might even get worse, but I'll be damned if it gets worse at the hands of anybody else. Get out and stay out.                              |
| Good        | Faith        | 1     | I can be well-mannered and well-meaning. After figuring out how to survive, figuring out how to live might just mean being kind to others. Social good is also the highest application of technology, which I have so dearly dedicated myself to in my studies. Sometimes, being nice backfires, and I go back into my shell, but might as well enjoy life until then.          |
| Inspiration | Beauty       | 5     | Sacred geometry haunts my dreams. I know that one day, somehow, I must realize it. Sculpt it. It is the closest thing I have in my life to a true purpose.                                                                                                                                                                                                                      |
| Rhetoric    | Intelligence | 6     | Nothing beats obliterating someone in conversation, or game. Even if that person is in my head. And never existed. At a certain level of philosophical authorship, that kind of delusion loops back around to being accepted as a "dialogue," a valid form of argument. One day, people will see how I belong with these great thinkers. I will prove it. One debate at a time. |
| Speed       | Tune         | 2     | Wake up people. Things happen slowly, and then fast. I'm always ready for the fast part. When all the dominoes fall and all the pieces fall into place, I can decide quickly whether to stand on business or get the hell out of here. Whatever I do I'll have the jump on everybody else while doing it. I think. It's never actually happened.                                |
| Evil        | Sadness      | 3     | There's something inside of me that I don't like. A desire to act with no values. And yet, I use it too much. Is there any escape?                                                                                                                                                                                                                                              |

Conversation summaries:
1. Sell the dagger, if possible. Speed reminds him who he's talking to. Rhetoric tries not to get scammed.
2. Hagieph greeting. Rhetoric keeps the atheism moving. Cage admires the structure of their cult.
3. Street hustler. Evil makes mad dopaminergic moves and respects the business. Inspiration analyzes the geometry of the game.
5. Sell the dagger, if possible. Otherwise, sell the sabre. Rhetoric sees another business opportunity. Good wants to analyze the hero's goals.
6. Introduce REDACTED. Inspiration catches on to patterns of speech relating to Arterology and Geometry and wants to talk academics.

## Reka
Reka was on pilgrimmage and lost her two best friends. She is now searching for them desparately. It's been one day since she's arrived and filed a report with the guard, but she's still in a bit of a panic.

| Skill                | Category     | Level | Description                                                                                                                                                                                                                                                                |
| -------------------- | ------------ | ----- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Power                | Strength     | 4     | First pillar of the tragic hero. Never give up and always move with a clear goal. The heroes which fail at this lose their way at the siren's call or turn around at the Gates of Hell.                                                                                    |
| Soul                 | Faith        | 2     | Third pillar of the tragic hero. Know one's limits and never be corrupted by temptation. The heroes which fail at this are slowly broken down by the evil of the world. After all the goals are completed, those without a Soul will inevitably use their powers for evil. |
| Body                 | Beauty       | 4     | Second pillar of the tragic hero. Care and cherish the self and others in both medicine and art. The heroes which fail at this are unpresentable, sad, and eventually die of sickness.                                                                                     |
| Nervous system       | Intelligence | 3     | Retreat, sidestep, connive, scheme, react erratically and never let Them know your next move. Your values are impractical. It's ok to skimp on them every once in a while to move quickly and effectively.                                                                 |
| Integumentary system | Tune         | 4     | Be attuned and obsessed with all things sensory. Seek out sounds, colors, textures and chemicals that make you feel fuzzy.                                                                                                                                                 |
| Skeletal System      | Sadness      | 4     | Maybe.. sometimes.. it's ok to give into your intrusive thoughts? Love, your skeleton.                                                                                                                                                                                     |

Conversation summaries:
1. Get me to the desert, help me find my friends. Skeletal System appreciates the clear not-giving-a-shit lifestyle of the bandit. 
2. Don't get distracted. Soul tries to stop her from getting distracted by the prospect of being biographed.
3. Street huslter. Nervous System tried to move quickly. Body tries to judge the posture of the opponent- a secret indicator of board evaluation.
4. Buy a weapon. Power reminds her that it's ok to be clear about what she wants. Nervous System tries to manipulate the conversation in her favor.
6. Divination, help me find my friends. Body and Skeletal System get distracted while REDACTED tries to help.

## REDACTED
A fish-man from the south has forgotten his own name. All he knows is that he needs to retrieve a magic artefact from the depths of the desert.

| Skill   | Category     | Level | Description                                                                                                                                                                                                                                                                                                                                                                   |
| ------- | ------------ | ----- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Heart   | Strength     | 2     | Somewhere, deep down, you know not to stop at anything to.. What was it you do again? Whatever it is- please -just don't give up. Your body is surprisingly sturdy, bulky even. It is an instrument of this, this grand scheme, somehow. Take care of your body, and everything inside of it, so it will not fail you. Not again.                                             |
| Stars   | Faith        | 4     | You can look at the stars and see the links between them. You can look at the thoughts in your head and superimpose them onto the stars. And thus, the ideas in your head connect as if it was preordained. Sometimes, you may connect things that nobody understands, not even you. And sometimes, it may even be useful.                                                    |
| Planets | Beauty       | 4     | You can look at the planets and see their signs. You can see how they relate to the sand. You can read fates. You forgot how you're doing it, but you speak the truth with a voice above your own, allowing it through as if you were flexing a muscle, or urinating.                                                                                                         |
| Memory  | Intelligence | 1     | Dewdrops of memory are building up on the branches of your neural pathways. Every once in a while, they drop, and you may catch them. Images of the past. Good luck separating them from everything else you think up.                                                                                                                                                        |
| Rhythm  | Tune         | 5     | Everything has a frequency. Figure them out, write them down, dance to them. Jump perfect arcs. Go hunting before winter and sleep when the first snow falls. Fail to anticipate out-of-time changes in the people around you. Match the natural series of the universe.                                                                                                      |
| Future  | Sadness      | 5     | Dark, static-filled clouds of possibility roam the dilapidated stretches of your mind, picking up sadness as it smokes off fried neurons. When a cloud reaches capacitance, bolts of raw Future strike the brain. The Future appears suddenly, mercilessly, obviously and unexplainably to you. One day, you will see how you die, and it will dictate the rest of your life. |

Conversation summaries:
1. Silence. Memory has a silent existential crisis. Does he not look familiar because you don't know him or because of the memory loss?
2. Chosen one. Hagieph senses great power in REDACTED. Future helps him avoid the cult and Heart keeps him on track.
3. Street hustler. Future always suggests the perfect move after a tough dice check.
4. Introduce merchant. Memory helps recall some basic things, but doesn't seem aware of how relieving this should be. Planets looks at the grains of sand on the merchant's face and describes where he's been.
5. Divination. Stars can use a Divination technique to reveal that one of the friends are somewhere in East Breach, while the other is out in the desert. Future admits that divination can be inaccurate.

# Additional Notes
Street hustler: a chess match. The hustler adapts to the player's accuracy. Players who win against the hustler gain an amount of money they bet in for at the beginning. Additionally, two player's can play against each other. Dialogue for each character should be present for:
 - Making a threat
 - Making a retreat
 - Making a discovered attack
 - Making a promotion
 - Making a castle
 - Making a check
 - Making a double check
 - Making a discovered check
 - Being one move away from checkmate
 - Making checkmate

Merchant: Reka starts with enough money to purchase items she wants, and Benjamin cannot buy over her. Money becomes more a decorative mechanic for the purposes of the example in this way.

# Callbacks and Variable Name Rules
One purpose of this example is to serve as a template for other people's projects, with common rpg functions implemented on the engine.

Callbacks:
 - `kick <character?>` Remove the character from the conversation. If no character is provided, remove the character who last acted. For example, if character A selects a node with the `kick` callback, character A is kicked.
 - `exp <amount> <character?>` Give an amount of experience to a character by name. Same character argument logic as `kick`.
 - `give <item> <character?>` Give an item to a character by name. Same character argument logic as `kick`.
 - `givemoney <amount> <character?>` Give an amount of money to a character by name. Same character argument logic as `kick`.
 - `show <tag> <...attributes> <at position> <fade_time>` show an image. similar syntax to renpy. Limited to fade transition with an adjustable time.
 - `hide <tag>` hide an image. Limited to fade transition with an adjustable time.

Asks:
 - `dc <character> <skill> <dice_expression> <threshold>` dice check with a given expression (like `2d6`), skill bonus and threshold
 - `has_item <character> <item>` check if the character has an item in their inventory.
 - `get_money <character>` get the money of a particular character.


# Items and Equipment

TODO! Fill this out.

# Architecture
These examples are part of the design process for the [engine](/engine/).

Outside of the "core" engine, which handles dialogue flow, this program also needs:
 - <NetworkingUI>. Menu for connecting to/creating a server over the network.
 - <CharacterSelectionUI>. Menu for selecting a character once connected to a server.
 - <GameWorldUI>. Menu for navigating the main area menu and dialogue that spawns from it after character selection, using number keys to select choices. Referred to in the previous sections as "the menu".
 - <ItemMenuUI>. Menu for inspecting items and changing equipment after reaching the game world.
 - <SkillsMenuUI>. Menu for inspecting skills and leveling up.
 - <MessageQueue>. A short message queue. When a "message" is described above, it's pushed to the bottom of this queue and the other messages are pushed up, deleting old messages if the maximum capacity of 3 is reached.

And all the behind-the-scenes structures to power that also, including:
 - <NetworkHandler> Syncronizes game state in a server authoritative manner
 - <CharacterAllocator> Handles character selection requests,
 provides information on allocated characters and prevents multiple allocations
 of one character.
 - <Engine> (or engine wrapper) which takes a conversation, position, etc. and
 returns relevant information.
 - <Lobby> manages the state of which characters are currently engaged
 in which conversation.
 - <Server> accepts commands from clients and checks with the allocator
 and the lobby. Synchronizes data with clients over the network.
 - <Client> sends commands to the server and listens for replies.
 - <CharacterEntity> An object representing a character in the scene. Handles logic
 for skill leveling, inventory items, and money.

# Server Logic
How the server accepts and rejects requests. 

The client sends:
 - Character selections
 - Skill levelups
 - Equipment changes
 - Dialogue entering
 - Dialogue exiting
 - Dialogue choices

All clients are then repeated these signals by the server, including an error
code indicating if the request passed the server logic. Clients only update their
state when the repeated signal is received.

There are also query signals. Generally, this game hides information through the
UI, not through the server, so the client can ask for information such as:
 - List of connected clients
 - Get characters of connected client
 - Get dialogue conversation and position of connected clients
 - Read story state variables
 - Read character state: inventory, skills, and money


## Character Selection
<Server> passes the request to <Lobby>. If the character is already selected,
<Lobby> declines. Otherwise, <Lobby> assigns the character to the client.

While this isn't needed for the demo, the lobby should not prevent multiple
characters from being allocated to the same client. This is instead enforced by
the UI flow.

## Skill Leveling
<Server> checks the source of the request against the <Lobby>. If the skill being
leveled is available among the characters allocated to the client making the request,
the request is passed to the appriopriate <CharacterEntity>.

## Equipment Changes
<Server> checks the source of the request against <Lobby> and the appropriate <CharacterEntity>. 
If the equipment is among the inventories of the characters allocated to the client making the request,
that equipment piece is swapped.

## Dialogue Entering/Exiting
<Server> passes the request to <Lobby>. <Lobby> checks if the associated characters
can enter/leave the requested conversation.

## Dialogue Choices
<Server> passes the request to <Engine>. This may cause multiple signals to be
repeated depending on the behavior of the engine. Typically, at the very least
a hub appearance event is repeated to all clients. 

These dialogue events also update state on the <CharacterEntity> objects on the
<Server>. These will not cause an immediate update over the network, but <Client>
s will read this state using their character state readers when displaying menus
later.

The `show` and `hide` callbacks also send signals to the clients when
processed, to control the display of images.

`kick` is handled by the server then causes a Dialogue exiting signal.

## Polling
The clients also poll the server to maintain their presence, getting dropped and
losing their character otherwise.

# Implementation Order
The most core mechanics are-
 - networking
 - character selection
 - dialogue
These should be implemented just with code first, then tested, then connected to
an interface.

Items and skills are desirable eventually but add a lot of unnecessary complexity.
If and only if networking, character selection, and dialogue are robust enough
to tolerate:
 - random disconnects
 - invalid requests (changing character mid conversation, leaving a conversation in which a character is an active participant, acting as a character other than the character selected, invalid choices, see Server Logic)

# Desireable result
The Louder engine should be a drop-in solution for all things dialogue. It exposes
functions for selecting dialogue choices by index, which is the same approach used
by the dialogue interface described in earlier sections. It can manage multiple
conversations on the server at the same time, and the server can send all relevant
information about dialogue to the clients, while translating the callbacks
described in Callbacks and Variable Name Rules into special "game events".

If there's a place where the engine cannot do this, with the first step of
implementation especially, that is a flaw of the engine that needs to be
addressed.