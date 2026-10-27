# Adapters

The [engine](../index.md) functionality is exposed through a variety of
"adapters": language bindings that also serve the purpose of automatically handling encoding, and
converting strings so the user doesn't have to interact with name ids directly.

# Rust Library
This is not an adapter, this is just the name of the central engine codebase.

`louder_rust` is the source code of the engine, and exposes many useful data
types. The docstrings in this code also serve as the authoritative documentation
on the function of the engine.

Strings cross this boundary as bytes. The engine stores names as the bytes it
was given and hands those same bytes back, and it decodes a document only as far
as JSON requires, which means UTF-8. It does not transcode anything and has no
encoding parameter to set: a host encodes what it hands over and decodes what it
gets back.

# Godot Bindings
`louder_godot` is a GDExtension built on [godot-rust](https://godot-rust.github.io).
It declares a Godot class over each engine type and is loaded by a
`.gdextension` file beside it.

# Python Bindings
Python bindings via [PYO3](https://github.com/PyO3/pyo3). This is the shape the
next adapter is expected to take: a crate in the same workspace, depending on
`louder_rust`, declaring its own classes and nothing else.
