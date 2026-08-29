# Architecture

!TODO! This is a theoretical architecture that I'm putting together as I draft
the engine in gdscript, eventually want to rewrite in a compile language for
better portability.

The engine is an event-driven system which supports primarily read-only
operations on events out from the engine. Here are the proposed event types.

event_start - a conversation has started in a given conversation at a given
position.

event_position - the engine's position in a given conversation has changed.

event_branch - the flow structure of a given conversation has changed branch a
has split off from branch b, or merged into it.

event_generic - emitted alongside any of the other events.

The state of the engine can also be polled. Given a conversation, whether that
conversation is active, the current position, branch structure, and data about
any position can be asked at any time safely thanks to atomic operations.