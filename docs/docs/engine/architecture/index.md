# Architecture

The engine is a discrete event system describing [actors](./actors.md) inside of a [conversation](./conversation.md), which itself is a network of [hubs](./hub.md), representing dialogue data created in the [editor](/editor). There is a [simulator](./simulator.md) which handles interactions between bodies and a conversation, a serializable [story_state](./story_state.md), and is responsible for emitting [events](./conversation.md) based on actions requested by bodies, and finally an [api](./api.md) which exposes all functionality described in [engine](/engine)

A normal, singleplayer dialogue tree would involve one of all of these objects,
while a multiplayer tree is intended to be achieved with a server-authoritative
instance of the simulator and conversation with multiple bodies connected.

In addition to these primary types, there is a set of supporting types:
 - [variant](./variant.md). One unit of data in a body, hub, or story_state.
 - [name](./name.md). A string temporarily mapped to an integer.
 - [entity](./entity.md). Base data for bodies, hubs, and story states.
 - [snapshot](./snapshot.md). A story_state, conversation, and set of bodies
 representing one entire dialogue state. 