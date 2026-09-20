# Cursed Godot Plus Renpy Client

A short demo about walking your dog. Demonstrates godot [adapters](../engine/adapters)
and stretches integration for a simple renpy conversation also.

Demo will consist of a renpy conversation whose progress is being synchronized
over the network with Godot. I don't plan on having full renpy support. But this
page on [statement equivalents](https://www.renpy.org/doc/html/statement_equivalents.html)
should be enough to get the job done for this purpose.

Player 2 in this demo will pilot the world around Player 1, throwing thunder, cars
crashing airplanes, zombies, and so on at Player 1 until the end of the walk.

If not interrupted, Player 1 goes through a set of basic locations without much
issue. If interrupted, Player 1 must deal with what is getting thrown their way
in a back-and-forth with Player 2.

If Player 2 is a human, they will interact through a renpy client looking down
on Player 1. If they are a bot, no additional client is needed. Player 1 acts
as the server and owns the actual simulation of dialogue.

When renpy recieves a signal that the dialogue state has changed from the server,
that signal is converted into the appropriate statement equivalents to display
text and images on Player 2's screen in ADV style. Player 2 should always have
choices in this demo until the very end, when either Player 1 dies or makes it
to the end of the walk.