use godot::prelude::*;

pub mod test_linkage;

struct LouderGDExt;

#[gdextension]
unsafe impl ExtensionLibrary for LouderGDExt {}

#[derive(GodotClass)]
#[class(base=Node)]
struct TestLinkage {
  base: Base<Node>
}

#[godot_api]
impl INode for TestLinkage {
  fn init(base: Base<Node> ) -> Self {
    godot_print!("Hello World from TestLinkage constructor!");

    Self {
      base,
    }
  }
}

#[godot_api]
impl TestLinkage {
  #[func]
  fn say_hello() {
    godot_print!("Hello World from TestLinkage custom function!");
  }
}
