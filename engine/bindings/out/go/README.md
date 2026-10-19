# Louder, from Go

Generated. Do not edit; run `python engine/bindings/generate.py` instead.

## Files

| File | What it is |
| --- | --- |
| `louder/louder.go` | The package: an owning `Louder`, real strings, and a `range` loop per cursor. |
| `louder/louder.h` | The C header cgo includes. |
| `cmd/walk/main.go` | A complete example. |

## Building

The package uses cgo, so it needs a C compiler. On Windows that is
MinGW gcc; the import library is copied into the package, so it links
as it ships:

```
go build ./...
```

To run, `louder_rust.dll` has to be on `PATH` (or `louder_rust.so` /
`louder_rust.dylib` on `LD_LIBRARY_PATH`).

## The bits worth knowing

- **`louder.Open()` returns an error.** The engine can refuse to start.
- **`Close` is explicit.** There is no finalizer: the engine cannot be
  closed from the garbage collector's thread.
- **Names are `int32` handles.** `l.Name(text)` does lookup-then-declare,
  because the engine refuses to make a second id for a string that has
  one. Handles come from a hash, so they are not sequential.
- **Every cursor is a loop.**

```go
for hub := range l.HubsByChannel(conversation, channel) {
    // ...
}
```

