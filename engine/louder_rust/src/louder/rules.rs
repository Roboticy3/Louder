bitflags::bitflags! {
  /// The dialogue rules a host can turn on and off.
  ///
  /// Both are on by default, because each is what makes the engine usable
  /// without every conversation having to be written around its absence. They
  /// are gathered here rather than left as separate switches so that a host
  /// reads and writes the whole configuration in one call, and so that adding a
  /// rule does not add another pair of engine calls.
  #[repr(C)]
  #[derive(Clone, Copy)]
  pub struct DialogueRules: i32 {
    /// The spoke rule. A lone choice on a hub may be taken by an actor who
    /// neither speaks nor hears the voice behind it, since there is nobody else
    /// to hand it to and no branch for it to be lost on. Without it, linear
    /// dialogue hands the lead back and forth between every actor instead of
    /// letting one of them carry it.
    const SPOKE = 0b00000001;

    /// The singleplayer rule. A hub that only the actor who just chose can see,
    /// reached from a hub that had several choices, resolves on its own instead
    /// of being shown. How many choices it carries is beside the point: it is
    /// the consequence of a choice rather than one, so there is nothing in it
    /// for that actor to decide. This is the "continue" of a more traditional
    /// singleplayer dialogue system, made implicit, so that a private branch
    /// can be routed through without being written as a decision.
    const SINGLEPLAYER = 0b00000010;
  }
}