use mod object::HomogeneousObject;
use mod object::LouderName;
use mod hub::LouderPosition;

//Interface for the contents of a conversation
pub struct Conversation {
  name: LouderName,
  hubs: Map<LouderPosition, Hub>,
  additional_data: HomogeneousObject,
}