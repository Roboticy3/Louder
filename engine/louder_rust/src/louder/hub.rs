use super::name::LouderName;
use super::object::HomogeneousObject;
use super::expression::{LouderExpression};
use super::callback::{LouderCallback};

bitflags::bitflags! {
  /// Flag bits describing a hub.
  #[repr(C)]
  #[derive(Clone)]
  pub struct HubFlags: i32 {
    const AUTO = 0b00000001;
    const ASK = 0b00000010;
  }
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
  choices: Vec<Choice>,

  expression: LouderExpression,
  callback: LouderCallback,
  flags: HubFlags,
  
  additional_data: HomogeneousObject,
}

pub struct Choice {
  text: LouderText,
  position: LouderPosition,
}