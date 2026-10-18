Scripts that reference nodes use @export and assign the references using scene files. If
referencing a node gets too complex for this method, use groups.

Singletons should be avoided without very good reason and existing singletons should be
simple.

# Running tests

Anything in this project can be tested empirically, so do not hand back GDScript
because there was no way to run it. Write a test as a plain Node script, name it
`test_*.gd`, and run it:

```
.\run_test.ps1 test_louder.gd
```

`run_test.ps1` builds `louder_godot` from the `engine/` workspace, copies the
library into `Louder/target/debug` where `Louder.gdextension` looks for it, and
runs the script headlessly. `test_louder.gd` is the example; new tests are new
files, not additions to it. Several can run in one go, in the order given:

```
.\run_test.ps1 test_louder.gd test_dialogue.gd
```

Everything after the script names is passed to the runner, which takes
`--frames N` for how many frames each script gets. The default of 2 is enough for
a script that does its work in `_ready`; a script that awaits `process_frame` or
needs several `_process` passes asks for more:

```
.\run_test.ps1 test_dialogue.gd --frames 60
```

## What the runner does

Godot only runs a script given to `--script` when that script extends SceneTree or
MainLoop, which rules out the Node scripts tests are written as, so `test_runner.gd`
starts them on the test script's behalf. It is the whole harness:

```
godot --headless --path . --script res://test_runner.gd -- res://test_louder.gd
```

That works on its own as long as the library has already been copied in, and it is
what `run_test.ps1` runs. The runner adds each script to the tree one at a time,
gives it the frame budget, frees it, and moves to the next, printing a `---` header
per script. Do not add tests to it: to add a test, add a script and pass its path.

## Writing a test

- Extend `Node`. `_ready` runs as the script is added, and `await`ing
  `get_tree().process_frame` and `_process` behave as they do in the game.
- `res://` paths reach the project's own files, so fixtures such as
  `res://conversations/jail.json` load the way they do at runtime.
- Print what is being checked, so a failure can be read out of the log.
- A test that fails calls `get_tree().quit(1)`. Godot stops handing out frames as
  soon as anything quits, so that exit code is what the process returns, what
  `run_test.ps1` returns, and the scripts queued behind it never start. A run that
  only prints exits 0. The runner also exits 1 itself when a script cannot be
  loaded or is not a Node script.
- To parse a script without running it:
  `godot --headless --path . --check-only --script res://test_louder.gd`

## After changing the engine

`run_test.ps1` rebuilds the extension every time, so changes under either
`engine/louder_rust` or `engine/louder_godot` are picked up by the next run with
no extra step. `build_and_copy_target.ps1` builds and copies the same way and then
opens the editor, which is for working on the project by hand rather than for
running tests.

## Registering a method

The classes come from `engine/louder_godot`, which wraps the engine types in
Godot classes of its own. A method is registered under the name of the Rust
function it came from, so the function is named what GDScript should call it.
Do not write `#[func(rename = "...")]`: godot-rust reads that value as a token
stream, so the string literal arrives carrying its own quotes and the method ends
up registered under `"add_actor"`, which the script editor shows in quotes and
only `call` can reach. `test_names.gd` fails on exactly that, so run it after any
change to the adapter.
