# Text Based CRPG

The purpose of this example is to provide the simplest possible demo which
takes advantage of all the engine's features:
 - Linking to other programming languages
 - Multiplayer conversations
 - Multiple conversation loading
 - Complex branching conversations

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
 8. Check equipment
 9. Check skills

Each of these opens a short conversation and sends the message "blank and blank start talking to each other" to all players. If Geno speaks to Reka, that will open
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

Reaching key points in conversations will give a player experience points, which
become levels in the skills menu. When the player gains experience, they will
get a message saying "x points to next level up." or "Level point gained!". They
can then enter the Check Skills menu to see "x points to next level up." or 
"x level points to allocated." plus another menu of skills. Selecting a skill
will show its description and the same level points message. If level points are
available, an option will be available in the skill description menu to allocate
a point to this skill.

Certain points in conversations will initiate a callback for a "dice check",
where a 2d6 will be rolled, and a skill's level added, then checked against a 
threshold before the result is sent back to the engine.

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
Street hustler: a chess match. The hustler adapts to the player's accuracy. Additionally, two player's can play against each other. Dialogue for each character should be present for:
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

# Callbacks and Variable Name Rules
One purpose of this example is to serve as a template for other people's projects.

Callbacks:
 - `kick <character>` remove the actor for a player-character from the conversation
 - `show <tag> <...attributes> <at position> <fade_time>` show an image. similar syntax to renpy. Limited to fade transition with an adjustable time.
 - `hide <tag>` hide an image. Limited to fade transition with an adjustable time.

Asks:
 - `dc <character> <skill> <dice_expression> <threshold>` dice check with a given expression (like `2d6`), skill bonus and threshold
 - `has_item <character> <item>` check if the character has an item in their inventory.
 - `get_money <character>` get the money of a particular character.

