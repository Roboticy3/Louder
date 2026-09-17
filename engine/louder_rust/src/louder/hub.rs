use super::object::{HomogeneousObject, LouderName, Map};
use super::expression::{LouderExpression};
use super::callback::{LouderCallback};

bitflags::bitflags! {
  /// Flag bits describing a hub.
  #[repr(transparent)]
  pub struct HubFlags: i32 {}
}

/// A position within a conversation network of hubs.
pub type LouderPosition = i32;

/// The text content of a hub.
pub type LouderText = String;

pub struct Hub {
  position: LouderPosition,

  speaker: LouderName,
  tags: Vec<LouderName>,
  recipients: Vec<LouderName>,

  channel: LouderName,
  priority: i32,

  text: LouderText,
  choices: Map<LouderText, LouderPosition>,

  expression: LouderExpression,
  callback: LouderCallback,
  flags: HubFlags,
  
  additional_data: HomogeneousObject,
}