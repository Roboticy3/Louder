use godot::prelude::*;

pub mod test_linkage;

struct LouderGDExt;

#[gdextension]
unsafe impl ExtensionLibrary for LouderGDExt {}
