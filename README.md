Multiplayer Dialog Experiment in Godot.

# Roadmap:
 [x] basic interface
 [x] serialization
 [ ] simulation
 [ ] portability
 [ ] multiplayer

# Goal:
Current interactive fiction tools like twine and yarn rely on a single-point
simulation where the player inhabits one part of a dialogue tree. I had an idea
for a game that required multiple players to be able to navigate a shared,
scripted dialogue tree and wasn't able to eek that out of the current tools.

While it's definitely possible to get that functionality eventually, I was also
not happy with the workflow in existing tools in general, and wanted something
I had more control over so I could build a faster UI.

My stretch goal is to implement the simulator in a way that it's portable- not
just to other godot projects but around the web and such, maybe in a web server
format so other programs can query it and run it in the background, or import
the raw javascript as a library. But that would be very unfamiliar territory for
me so I'm going to make sure the simulation works in Godot world first.