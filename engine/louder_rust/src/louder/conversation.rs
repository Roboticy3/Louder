use super::object::{HomogeneousObject, LouderName, Map};
use super::hub::{Hub, LouderPosition};

//Interface for the contents of a conversation
pub struct Conversation {
  name: LouderName,
  hubs: Map<LouderPosition, Hub>,
  additional_data: HomogeneousObject,
}