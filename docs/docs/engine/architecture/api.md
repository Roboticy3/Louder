# API

The set of functions revealed to the user.

The authoritative reference for every function, type, and constant is the
[API Reference](./api_reference.md), generated directly from the engine's Rust
source. This page describes the concepts and conventions shared across the API.

# Names

When this api refers to a "name", it's referring to an integer in this system.

Conversations are id'd by names containing the file they were loaded from.

# Generic Errors

Basic errors that are reused in multiple contexts.

`LouderError`
 - 0: no error
 - -1: does not exist

# Events

The engine reports what happens through a queue of events. Poll events by index
until one no longer exists, then flush.

Intended usage:
```c
int i = 0;
while (is_event(ctx, i)) {
  // poll event properties...
  i++;
}
flush_events(ctx);
```

An event's type flags (`get_event_flags`) determine which other properties it
has. The full list of flags and event property keys is in the
[API Reference](./api_reference.md). Events with no flags are invalid and
shouldn't appear.

Generally, language wrappers should split these events into individual events
for the different types of events described in the requirements.

# Accessors

"read" methods are in buffer form (`int read_...(... type* buffer, int
max_count)` = # of items written to buffer), taking "max_" size arguments and
mutable pointer args.

Iterator methods are written in the form `next_...` and take a start argument
which should default to -1.

# Modifiers

Functions that change the state of the engine: loading and removing
conversations, updating the config, adding and moving actors, answering asks,
and setting story state values.