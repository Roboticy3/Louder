/* Bindings for C, generated from the C header of the Louder engine.
 *
 * One difference from the header the engine ships: this one compiles. C keeps
 * struct tags, typedef names, and enum constants in a single namespace, and the
 * engine has a variant of `EventData_Tag` named `ChoiceAppearance` alongside a
 * struct of the same name. A constant that would redeclare something else in the
 * file is spelled with the enum's name in front of it; nothing else is renamed.
 *
 * `louder.c` holds the convenience layer: a growable buffer, the string
 * arguments as `const char *`, and a walk for each of the cursors.
 *
 * Do not edit. Run `python engine/bindings/generate.py` and this file comes back.
 */

/*
 * Nothing is defined here on purpose. The convenience layer is `static inline`
 * in the header, so a host needs only to include it. This translation unit
 * exists so that compiling it is a check on the header.
 */

#include "louder.h"
