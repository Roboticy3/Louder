# Engine

The Louder engine is an open-source program meant to interpret conversations
created by the [editor](/editor/). The engine can manage, save, and load state
for conversations, manage the flow of conversation, and save and load replays
and checkpoints of a conversation.

This is very much a back-end tool, and does not help with rendering text, sound,
character portraits, etc. This deficiency is the purpose of the [examples](/examples/)
secion.

## Basic Simulation Rules
Dialog is loaded from a project file produced by the editor (see [editor](/editor/) File Formats section) by the "simulator", the object responsible for simulation. This loaded conversation object can then be used to start a "session." A normal session is described below.

### Normal Session
!TODO! Rename Expressions to Statements and group Conditions and Statements under "Commands". Also, the last rule in 7 about choice text being used in absence of text content is not yet implemented. [32](https://github.com/Roboticy3/Louder/issues/32)

 1. Find a node with the "start" property set to true. !TODO! A starting node can also be selected by passing in specific tags. The first node with matching substrings in their "Tags" property will be used. [33](https://github.com/Roboticy3/Louder/issues/33)
 3. The "Statements" property of the starting node is evaluated (see Commands). 
 4. A choice can be selected from the choices available on the starting node. Each choice can correspond to another node in the conversation. Choices are filtered by whether the node they point to exists, and whether the "Conditions" property of the node is evaluated to true or false. As long as each choice corresponds to at most one node, this is an unambiguous decision. Choices can be selected by numeric index by which they appear after filtering, or by their string contents.
 5. If the node of a choice has the "dice_skill" property filled by a valid Identifier with an Integer value, that value is added to a 2d6 dice roll. If this dice roll is greater than or equal to the node's "dice_level" property, the roll is treated as a true Condition, otherwise, it's treated as false and the choice becomes inaccessible.
 6. Once a choice is selected, the simulator travels to the associated node. If the starting node has the "auto" property set to true, the choice index 0 is selected automatically if available.
 7. When the simulator travels to another node, the process repeats, either waiting for more choices from the user to proceed or proceeding automatically as long as the simulator is traveling through "auto" nodes. If the node has a blank "text" field, the text of the previous choice is used.

### Commmands
!TODO! Not all assignment expressions are currently supported. Only = is currently supported [31](https://github.com/Roboticy3/Louder/issues/31)

!TODO! Currently there is no error logging. [34](https://github.com/Roboticy3/Louder/issues/34)

The "Statements" and "Conditions" properties of nodes are both types of commands- blocks of code, essentially, broken up into individual lines. Conditions allow for computing a value from certain variables. 

Here are the syntax elements for commands:

| Element | Name | Availability | Description | Example | 
|---|---|---|---|---|
| name starting with a-z, A-Z or _ and containing only a-z, A-Z, 0-9 or _ | Identifier | Conditions and Expressions | The name of a variable. If the variable has not been defined, it is given a default value of 0 | `x` or `_awesome` are Identifiers by `2head` is not | 
| +,-,*,/,%,(,) | Basic Math | Conditions and Expressions | Basic math symbols. In order of priority: Addition + Subtraction - Multiplication * Division / Modulo by an Integer % Open Group ( Close Group ) | `(x + y) * z + w` is the sum of x,y, multiplied by z, then added to w | 
| =,+=,-=,++,\-\- | Assignment | Expressions only | Assignment operators. Must have an Identifier on the left and an Expression on the right. = Assigns the value of Expression to Identifier. +=, -= Add/Subtract the value of Expression from Identifier. ++/-- Increment/decrement the Identifier by 1. | `not_yet_defined_ident++` will define `not_yet_defined_ident` with the value 1 | 
| \\n (newline) | End | End of a command | In the editor, multiple commands can be stacked using multiple lines of code | 

Here are the types of data which can be stored by an identifier. Types are treated loosely, meaning they can change without causing errors:

| Type | Description | Example | 
| --- | --- | --- | 
| Integer | A number like -1, 0, or 1. Division or Modulo by 0 causes an [error](/engine#Logging). Dividing Integers by Integers takes the floor of the resulting fraction. | `x % 10` evaluates | 
| Float | A decimal number like 0.5. Arithmetic between Floats and Integers produces floats. Note that 0.0 is a Float and adding 0.0 to the Integer 1 will produce the Float 1.0. Just like Integers, Division or Modulo by 0 causes an [error](/engine#Logging) | `y / 0.5` | 
| String | A sequence of utf-8 characters such as "abcd". Adding strings concatenates them. | `y = "a" + "b"` will define the Identifier `y` with the value "ab" | 
| Boolean | true or false | `x = true` sets `x` to true and `x = false` sets it to false | 

Expressions always _evaluate_ to a value. The following values are treated as false: empty string `""`, zero `0` or `0.0`, and of course the Boolean `false`. All other values are treated as "truthy" for the purposes of the "Conditions" property in simulation.

### Cheats
!TODO! Manual command sending, saving and loading are not yet implemented on the prototype simulator [30](https://github.com/Roboticy3/Louder/issues/30)

The user can also manually manipulate the simulation outside of normal flow. This includes manually warping to a specific node with or without progressing through auto nodes, and running commands to either see what they evaluate to or manipulate the values of variables. The ability to read and write from the conversation state using commands is particularly useful for linking the simulation with the rest of a larger game, and the current state of the conversation can be saved out
to a list of newline separated commands which can then be replayed to reload a certain state.

## Advanced Simulation Rules

### Synchronous Multiple Speakers
One way to approach multiple playable characters in dialogue is to first have a set of rules, where each speaker can "own" nodes belonging to a list of speakers, indicated by the "Speaker" property. Next, for 2 or more clients, choices are restricted by those that lead to nodes either in the own list.

Additionally, speakers can be flagged as "Private". When dialogue enters a node owned by a private speaker, clients that do not own that speaker percieve nothing as happening. They can also check for other player-characters other than their own with access to the node, displaying "<character> is thinking" in these sections.

Only if *all* clients have no remaining dialogue options does the simulator send an "end" signal to all clients.

The engine provides functions to tell a client who, if anyone, is thinking. And the effect of these rules on the basic rules is otherwise subtractive- taking the original behavior of showing dialogue and choices and simply not showing them sometimes.