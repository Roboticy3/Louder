# Louder, from C

Generated. Do not edit; run `python engine/bindings/generate.py` instead.

## Files

| File | What it is |
| --- | --- |
| `louder.h` | The boundary. Every type, constant, and entry point, with the layout pinned by static assertions. |
| `louder.c` | The convenience layer, which a host can ignore entirely. |
| `louder_rust.dll` | The engine itself. |

## Using it

```c
#include "louder.h"

int main(void) {
    LouderContext *engine = init_louder();
    LouderName conversation = louder_name(engine, "scene");

    LouderBuffer json = {0};
    if (!louder_read_state_json(engine, conversation, &json)) {
        louder_buffer_free(&json);
        return 1;
    }
    puts((const char *)json.data);
    louder_buffer_free(&json);

    deinit_louder(engine);
    return 0;
}
```

## Compiling

```
gcc -I. walk.c louder.c -L. -llouder_rust -o walk
```

## The bits worth knowing

- **Names are integers.** `LouderName` is an `int32_t` handle into the
  engine intern table. `louder_name` declares text and returns the handle;
  `louder_read_name` reads it back. Handles come from a hash, so they are
  not sequential.
- **Text crosses as bytes.** A `uint8_t *` and a `size_t`, not a
  `const char *`. The engine expects UTF-8 in the JSON and gives back
  exactly the bytes that went in, so decoding is the host's job.
- **Buffers never fail by being small.** A read reports what fitted rather
  than refusing, so `louder_read_name` and friends ask again with more
  room. Write your own loop the same way if you call the header directly.
- **A tagged union is a tag and a body.** `LouderVariant.body.int_` is
  only meaningful when `LouderVariant.tag` says so.
- **Every cursor ends at a value you pass in.** `louder_hubs_by_channel`
  and the rest take both the value to start from and the value that means
  stop, because the header declares that constant for some and not others.

## One difference from the header the engine ships

This one compiles. C keeps struct tags, typedef names, and enum constants
in a single namespace, and the engine has a variant of `EventData_Tag`
named `ChoiceAppearance` alongside a struct of the same name. A constant
that would redeclare something else in the file is spelled with the enum's
name in front of it, so it is `EventData_Tag_ChoiceAppearance`. Nothing
else is renamed.

