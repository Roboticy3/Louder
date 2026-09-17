use mod object::HomogeneousObject;
use mod object::LouderName;
use mod object::Map;

use mod expression::LouderExpression;
use mod callback::LouderCallback;

/// Flag bits describing a hub.
pub type HubFlags = i32;

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