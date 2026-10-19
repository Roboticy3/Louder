// Bindings for JavaScript, generated from the C header of the Louder engine.
//
// Do not edit. Run `python engine/bindings/generate.py` and this file comes back.
/**
 * The C boundary, as the WebAssembly module actually exports it.
 *
 * There is no glue code between here and the engine. The module is
 * instantiated directly and its exports called with the arguments its own
 * type section says they take, which is where the layouts below and the
 * signatures in this file come from.
 */

import {
  fromBytes,
  put,
  readInto,
  reserve,
  take,
  toBytes,
  view,
  withBytes,
} from "./memory.js";


/** How much of linear memory each declared type occupies. */
export const SIZE = Object.freeze({
  ActorFlags: 4,
  AdditionalDataChange: 4,
  ChoiceAppearance: 8,
  ConditionOutcome: 12,
  DialogueRules: 4,
  EventData: 20,
  EventData_Body: 16,
  EventData_Tag: 4,
  HubFlags: 4,
  LoadError: 4,
  LoadFailure: 8,
  LouderActor: 4,
  LouderAsk: 32,
  LouderCallback: 4,
  LouderError: 4,
  LouderEvent: 32,
  LouderExpression: 4,
  LouderName: 4,
  LouderOption_LouderAsk: 36,
  LouderOption_LouderAsk_Body: 32,
  LouderOption_LouderAsk_Tag: 4,
  LouderOption_LouderEvent: 36,
  LouderOption_LouderEvent_Body: 32,
  LouderOption_LouderEvent_Tag: 4,
  LouderPosition: 4,
  LouderVariant: 8,
  LouderVariant_Body: 4,
  LouderVariant_Tag: 4,
  Motion: 16,
  SpeakerFlags: 4,
});

/**
 * Where each field sits inside its type.
 *
 * Every arm of a tagged union shares one block of memory, so they are all
 * read from the same offset and the tag says which one means anything.
 */
export const OFFSET = Object.freeze({
  ActorFlags: Object.freeze({
    bits: 0,
  }),
  AdditionalDataChange: Object.freeze({
    key: 0,
  }),
  ChoiceAppearance: Object.freeze({
    position: 0,
    choice_index: 4,
  }),
  ConditionOutcome: Object.freeze({
    condition: 0,
    outcome: 4,
  }),
  DialogueRules: Object.freeze({
    bits: 0,
  }),
  EventData: Object.freeze({
    tag: 0,
  }),
  EventData_Body: Object.freeze({
    regular_motion: 4,
    warp: 4,
    force_motion: 4,
    choice_appearance: 4,
    additional_data_change: 4,
    speaker_change: 4,
    text_change: 4,
    statement: 4,
    condition: 4,
    callback: 4,
    speaker_added: 4,
    speaker_removed: 4,
    conversation_added: 4,
    conversation_removed: 4,
    load_error: 4,
  }),
  HubFlags: Object.freeze({
    bits: 0,
  }),
  LoadFailure: Object.freeze({
    position: 0,
    error: 4,
  }),
  LouderAsk: Object.freeze({
    conversation: 0,
    channel: 4,
    actor: 8,
    motion: 12,
    ask: 28,
  }),
  LouderEvent: Object.freeze({
    conversation: 0,
    channel: 4,
    actor: 8,
    data: 12,
  }),
  LouderOption_LouderAsk: Object.freeze({
    tag: 0,
  }),
  LouderOption_LouderAsk_Body: Object.freeze({
    some: 4,
  }),
  LouderOption_LouderEvent: Object.freeze({
    tag: 0,
  }),
  LouderOption_LouderEvent_Body: Object.freeze({
    some: 4,
  }),
  LouderVariant: Object.freeze({
    tag: 0,
  }),
  LouderVariant_Body: Object.freeze({
    int_: 4,
    float_: 4,
    string: 4,
  }),
  Motion: Object.freeze({
    source_position: 0,
    target_position: 4,
    choice_index: 8,
    background: 12,
  }),
  SpeakerFlags: Object.freeze({
    bits: 0,
  }),
});

/** Tag value to arm name, for every tagged union. */
export const KINDS = Object.freeze({
  EventData: Object.freeze({
    0: "None",
    1: "RegularMotion",
    2: "Warp",
    3: "ForceMotion",
    4: "ChoiceAppearance",
    5: "ChannelChange",
    6: "AdditionalDataChange",
    7: "SpeakerChange",
    8: "TextChange",
    9: "Statement",
    10: "Condition",
    11: "Callback",
    12: "ActorAdded",
    13: "ActorRemoved",
    14: "SpeakerAdded",
    15: "SpeakerRemoved",
    16: "ConversationAdded",
    17: "ConversationRemoved",
    18: "LoadError",
  }),
  LouderOption_LouderAsk: Object.freeze({
    0: "None_LouderAsk",
    1: "Some_LouderAsk",
  }),
  LouderOption_LouderEvent: Object.freeze({
    0: "None_LouderEvent",
    1: "Some_LouderEvent",
  }),
  LouderVariant: Object.freeze({
    0: "Int",
    1: "Float",
    2: "String",
  }),
});

/** An error code returned by most operations. - LouderError::Ok: no error. - LouderError::Missing: the requested object does not exist. */
export const LouderError = Object.freeze({
  Ok: 0,
  Missing: 1,
});

/** Tag name to tag value, for every tagged union. */
export const TAGS = Object.freeze({
  EventData_Tag: Object.freeze({
    None: 0,
    RegularMotion: 1,
    Warp: 2,
    ForceMotion: 3,
    ChoiceAppearance: 4,
    ChannelChange: 5,
    AdditionalDataChange: 6,
    SpeakerChange: 7,
    TextChange: 8,
    Statement: 9,
    Condition: 10,
    Callback: 11,
    ActorAdded: 12,
    ActorRemoved: 13,
    SpeakerAdded: 14,
    SpeakerRemoved: 15,
    ConversationAdded: 16,
    ConversationRemoved: 17,
    LoadError: 18,
  }),
  LouderOption_LouderAsk_Tag: Object.freeze({
    None_LouderAsk: 0,
    Some_LouderAsk: 1,
  }),
  LouderOption_LouderEvent_Tag: Object.freeze({
    None_LouderEvent: 0,
    Some_LouderEvent: 1,
  }),
  LouderVariant_Tag: Object.freeze({
    Int: 0,
    Float: 1,
    String: 2,
  }),
});

/** Why a document was refused. Zero means it was not refused. */
export const LoadError = Object.freeze({
  0: "Ok",
  1: "MalformedJson",
  2: "NotAList",
  3: "NotAnObject",
  4: "AlreadyLoaded",
});


// // Flag bits describing an actor.
export function readActorFlags(view, at) {
  return {
    bits: view.getInt32(at + 0, true),
  };
}

export function writeActorFlags(view, at, value) {
  view.setInt32(at + 0, value.bits, true);
}

// // A key of an actor's additional data being replaced.
export function readAdditionalDataChange(view, at) {
  return {
    key: view.getInt32(at + 0, true),
  };
}

export function writeAdditionalDataChange(view, at, value) {
  view.setInt32(at + 0, value.key, true);
}

// // A choice becoming available on a hub.
export function readChoiceAppearance(view, at) {
  return {
    position: view.getInt32(at + 0, true),
    choiceIndex: view.getInt32(at + 4, true),
  };
}

export function writeChoiceAppearance(view, at, value) {
  view.setInt32(at + 0, value.position, true);
  view.setInt32(at + 4, value.choiceIndex, true);
}


export function readLouderVariant_Body(view, at) {
  return {
    int: view.getInt32(at + 0, true),
    float: view.getFloat32(at + 0, true),
    string: view.getInt32(at + 0, true),
  };
}

export function writeLouderVariant_Body(view, at, value) {
  view.setInt32(at + 0, value.int, true);
  view.setFloat32(at + 0, value.float, true);
  view.setInt32(at + 0, value.string, true);
}

// // What a condition on a hub came to, when an actor landed on it.
export function readConditionOutcome(view, at) {
  return {
    condition: view.getInt32(at + 0, true),
    outcome: readLouderVariant(view, at + 4),
  };
}

export function writeConditionOutcome(view, at, value) {
  view.setInt32(at + 0, value.condition, true);
  writeLouderVariant(view, at + 4, value.outcome);
}

// // The dialogue rules a host can turn on and off.
// // 
// // Both are on by default, because each is what makes the engine usable without every
// // conversation having to be written around its absence. They are gathered here rather than
// // left as separate switches so that a host reads and writes the whole configuration in one
// // call, and so that adding a rule does not add another pair of engine calls.
export function readDialogueRules(view, at) {
  return {
    bits: view.getInt32(at + 0, true),
  };
}

export function writeDialogueRules(view, at, value) {
  view.setInt32(at + 0, value.bits, true);
}

// // One move of an actor from one hub to another.
// // 
// // Every event a move produces carries the same motion, which is how a host reads where an
// // actor came from and where it went.
export function readMotion(view, at) {
  return {
    sourcePosition: view.getInt32(at + 0, true),
    targetPosition: view.getInt32(at + 4, true),
    choiceIndex: view.getInt32(at + 8, true),
    background: (view.getUint8(at + 12) !== 0),
  };
}

export function writeMotion(view, at, value) {
  view.setInt32(at + 0, value.sourcePosition, true);
  view.setInt32(at + 4, value.targetPosition, true);
  view.setInt32(at + 8, value.choiceIndex, true);
  view.setUint8(at + 12, value.background ? 1 : 0);
}

// // A record in a document the engine could not read.
// // 
// // Carries the position of the record that was at fault, or LOUDER_POSITION_NONE when the
// // problem was with the document as a whole.
export function readLoadFailure(view, at) {
  return {
    position: view.getInt32(at + 0, true),
    error: view.getInt32(at + 4, true),
  };
}

export function writeLoadFailure(view, at, value) {
  view.setInt32(at + 0, value.position, true);
  view.setInt32(at + 4, value.error, true);
}


export function readEventData_Body(view, at) {
  return {
    regularMotion: readMotion(view, at + 0),
    warp: readMotion(view, at + 0),
    forceMotion: readMotion(view, at + 0),
    choiceAppearance: readChoiceAppearance(view, at + 0),
    additionalDataChange: readAdditionalDataChange(view, at + 0),
    speakerChange: view.getInt32(at + 0, true),
    textChange: view.getInt32(at + 0, true),
    statement: view.getInt32(at + 0, true),
    condition: readConditionOutcome(view, at + 0),
    callback: view.getInt32(at + 0, true),
    speakerAdded: view.getInt32(at + 0, true),
    speakerRemoved: view.getInt32(at + 0, true),
    conversationAdded: view.getInt32(at + 0, true),
    conversationRemoved: view.getInt32(at + 0, true),
    loadError: readLoadFailure(view, at + 0),
  };
}

export function writeEventData_Body(view, at, value) {
  writeMotion(view, at + 0, value.regularMotion);
  writeMotion(view, at + 0, value.warp);
  writeMotion(view, at + 0, value.forceMotion);
  writeChoiceAppearance(view, at + 0, value.choiceAppearance);
  writeAdditionalDataChange(view, at + 0, value.additionalDataChange);
  view.setInt32(at + 0, value.speakerChange, true);
  view.setInt32(at + 0, value.textChange, true);
  view.setInt32(at + 0, value.statement, true);
  writeConditionOutcome(view, at + 0, value.condition);
  view.setInt32(at + 0, value.callback, true);
  view.setInt32(at + 0, value.speakerAdded, true);
  view.setInt32(at + 0, value.speakerRemoved, true);
  view.setInt32(at + 0, value.conversationAdded, true);
  view.setInt32(at + 0, value.conversationRemoved, true);
  writeLoadFailure(view, at + 0, value.loadError);
}

// // Flag bits describing a hub.
export function readHubFlags(view, at) {
  return {
    bits: view.getInt32(at + 0, true),
  };
}

export function writeHubFlags(view, at, value) {
  view.setInt32(at + 0, value.bits, true);
}

// // A question the engine has asked the host and is waiting on.
// // 
// // Answering an ask moves it to the back of the cache, so indexes shift; read the cache
// // again after each answer.
export function readLouderAsk(view, at) {
  return {
    conversation: view.getInt32(at + 0, true),
    channel: view.getInt32(at + 4, true),
    actor: view.getInt32(at + 8, true),
    motion: readMotion(view, at + 12),
    ask: view.getInt32(at + 28, true),
  };
}

export function writeLouderAsk(view, at, value) {
  view.setInt32(at + 0, value.conversation, true);
  view.setInt32(at + 4, value.channel, true);
  view.setInt32(at + 8, value.actor, true);
  writeMotion(view, at + 12, value.motion);
  view.setInt32(at + 28, value.ask, true);
}

// // One entry in the event queue.
// // 
// // The queue is polled from index 0 upwards and emptied with
// // LouderContext::flush_events(super::context::LouderContext).
export function readLouderEvent(view, at) {
  return {
    conversation: view.getInt32(at + 0, true),
    channel: view.getInt32(at + 4, true),
    actor: view.getInt32(at + 8, true),
    data: readEventData(view, at + 12),
  };
}

export function writeLouderEvent(view, at, value) {
  view.setInt32(at + 0, value.conversation, true);
  view.setInt32(at + 4, value.channel, true);
  view.setInt32(at + 8, value.actor, true);
  writeEventData(view, at + 12, value.data);
}


export function readLouderOption_LouderAsk_Body(view, at) {
  return {
    some: readLouderAsk(view, at + 0),
  };
}

export function writeLouderOption_LouderAsk_Body(view, at, value) {
  writeLouderAsk(view, at + 0, value.some);
}


export function readLouderOption_LouderEvent_Body(view, at) {
  return {
    some: readLouderEvent(view, at + 0),
  };
}

export function writeLouderOption_LouderEvent_Body(view, at, value) {
  writeLouderEvent(view, at + 0, value.some);
}

// // Flag bits describing a speaker.
export function readSpeakerFlags(view, at) {
  return {
    bits: view.getInt32(at + 0, true),
  };
}

export function writeSpeakerFlags(view, at, value) {
  view.setInt32(at + 0, value.bits, true);
}

// // What an LouderEvent is about.
// // 
// // A tagged union cannot be read as one value by a host that has to switch on it, so this is
// // the discriminant of an EventData as an `int`, and the payload is read through the
// // accessors that match it. A host that does not recognise a value is looking at something a
// // newer engine produced.
export function readEventData(view, at) {
  return {
    tag: view.getInt32(at + OFFSET.EventData.tag, true),
    regularMotion: readMotion(view, at + 4),
    warp: readMotion(view, at + 4),
    forceMotion: readMotion(view, at + 4),
    choiceAppearance: readChoiceAppearance(view, at + 4),
    additionalDataChange: readAdditionalDataChange(view, at + 4),
    speakerChange: view.getInt32(at + 4, true),
    textChange: view.getInt32(at + 4, true),
    statement: view.getInt32(at + 4, true),
    condition: readConditionOutcome(view, at + 4),
    callback: view.getInt32(at + 4, true),
    speakerAdded: view.getInt32(at + 4, true),
    speakerRemoved: view.getInt32(at + 4, true),
    conversationAdded: view.getInt32(at + 4, true),
    conversationRemoved: view.getInt32(at + 4, true),
    loadError: readLoadFailure(view, at + 4),
  };
}

export function writeEventData(view, at, value) {
  view.setInt32(at + OFFSET.EventData.tag, value.tag, true);
  if (value.regularMotion !== undefined) {
    writeMotion(view, at + 4, value.regularMotion)
  }
  if (value.warp !== undefined) {
    writeMotion(view, at + 4, value.warp)
  }
  if (value.forceMotion !== undefined) {
    writeMotion(view, at + 4, value.forceMotion)
  }
  if (value.choiceAppearance !== undefined) {
    writeChoiceAppearance(view, at + 4, value.choiceAppearance)
  }
  if (value.additionalDataChange !== undefined) {
    writeAdditionalDataChange(view, at + 4, value.additionalDataChange)
  }
  if (value.speakerChange !== undefined) {
    view.setInt32(at + 4, value.speakerChange, true)
  }
  if (value.textChange !== undefined) {
    view.setInt32(at + 4, value.textChange, true)
  }
  if (value.statement !== undefined) {
    view.setInt32(at + 4, value.statement, true)
  }
  if (value.condition !== undefined) {
    writeConditionOutcome(view, at + 4, value.condition)
  }
  if (value.callback !== undefined) {
    view.setInt32(at + 4, value.callback, true)
  }
  if (value.speakerAdded !== undefined) {
    view.setInt32(at + 4, value.speakerAdded, true)
  }
  if (value.speakerRemoved !== undefined) {
    view.setInt32(at + 4, value.speakerRemoved, true)
  }
  if (value.conversationAdded !== undefined) {
    view.setInt32(at + 4, value.conversationAdded, true)
  }
  if (value.conversationRemoved !== undefined) {
    view.setInt32(at + 4, value.conversationRemoved, true)
  }
  if (value.loadError !== undefined) {
    writeLoadFailure(view, at + 4, value.loadError)
  }
}

// // Custom Option type exposed to the C FFI
export function readLouderOption_LouderAsk(view, at) {
  return {
    tag: view.getInt32(at + OFFSET.LouderOption_LouderAsk.tag, true),
  };
}

export function writeLouderOption_LouderAsk(view, at, value) {
  view.setInt32(at + OFFSET.LouderOption_LouderAsk.tag, value.tag, true);
}

// // Custom Option type exposed to the C FFI
export function readLouderOption_LouderEvent(view, at) {
  return {
    tag: view.getInt32(at + OFFSET.LouderOption_LouderEvent.tag, true),
  };
}

export function writeLouderOption_LouderEvent(view, at, value) {
  view.setInt32(at + OFFSET.LouderOption_LouderEvent.tag, value.tag, true);
}

// // One unit of data in an actor, hub, speaker, or story state.
// // 
// // Booleans are not a separate kind: they are carried as LouderVariant::Int 1 or 0, which is
// // what makes them interoperable with the arithmetic and comparison rules of the engine
// // guide.
export function readLouderVariant(view, at) {
  return {
    tag: view.getInt32(at + OFFSET.LouderVariant.tag, true),
    int: view.getInt32(at + 4, true),
    float: view.getFloat32(at + 4, true),
    string: view.getInt32(at + 4, true),
  };
}

export function writeLouderVariant(view, at, value) {
  view.setInt32(at + OFFSET.LouderVariant.tag, value.tag, true);
  if (value.int !== undefined) {
    view.setInt32(at + 4, value.int, true)
  }
  if (value.float !== undefined) {
    view.setFloat32(at + 4, value.float, true)
  }
  if (value.string !== undefined) {
    view.setInt32(at + 4, value.string, true)
  }
}
/** The constants the header declares, under the names it gives them. */
export const CONSTANTS = Object.freeze({
  ActorFlags_SPECTATOR: 1,
  ActorFlags_DEBUG: 2,
  DialogueRules_SPOKE: 1,
  DialogueRules_SINGLEPLAYER: 2,
  HubFlags_AUTO: 1,
  HubFlags_ASK: 2,
  SpeakerFlags_PUBLIC: 1,
  LOUDER_ACTOR_NONE: -1,
  LOUDER_LOAD_ALREADY_LOADED: 4,
  LOUDER_LOAD_MALFORMED_JSON: 1,
  LOUDER_LOAD_NOT_AN_OBJECT: 3,
  LOUDER_LOAD_NOT_A_LIST: 2,
  LOUDER_LOAD_OK: 0,
  LOUDER_NAME_NONE: -1,
});

export const ActorFlags_SPECTATOR = CONSTANTS.ActorFlags_SPECTATOR;
export const ActorFlags_DEBUG = CONSTANTS.ActorFlags_DEBUG;
export const DialogueRules_SPOKE = CONSTANTS.DialogueRules_SPOKE;
export const DialogueRules_SINGLEPLAYER = CONSTANTS.DialogueRules_SINGLEPLAYER;
export const HubFlags_AUTO = CONSTANTS.HubFlags_AUTO;
export const HubFlags_ASK = CONSTANTS.HubFlags_ASK;
export const SpeakerFlags_PUBLIC = CONSTANTS.SpeakerFlags_PUBLIC;
export const LOUDER_ACTOR_NONE = CONSTANTS.LOUDER_ACTOR_NONE;
export const LOUDER_LOAD_ALREADY_LOADED = CONSTANTS.LOUDER_LOAD_ALREADY_LOADED;
export const LOUDER_LOAD_MALFORMED_JSON = CONSTANTS.LOUDER_LOAD_MALFORMED_JSON;
export const LOUDER_LOAD_NOT_AN_OBJECT = CONSTANTS.LOUDER_LOAD_NOT_AN_OBJECT;
export const LOUDER_LOAD_NOT_A_LIST = CONSTANTS.LOUDER_LOAD_NOT_A_LIST;
export const LOUDER_LOAD_OK = CONSTANTS.LOUDER_LOAD_OK;
export const LOUDER_NAME_NONE = CONSTANTS.LOUDER_NAME_NONE;

/** Every entry point, called the way the module says it is. */
export class Raw {
  constructor(instance) {
    this.exports = instance.exports;
    this.memory = instance.exports.memory;
  }

  /** A view over linear memory, current as of this call. */
  bytes() {
    return view(this.memory);
  }

  add_actor(ctx, conversation, startPosition, flags, speakAs, speakAsCount, listenTo, listenToCount) {
    let view;
    const flagsAt = reserve(this.memory, 4);
    view = this.bytes();
    writeActorFlags(view, flagsAt, flags.bits);
    return this.exports.add_actor(ctx, conversation, startPosition, flagsAt, speakAs, speakAsCount, listenTo, listenToCount);
  }

  answer_ask(ctx, index, verdict) {
    return this.exports.answer_ask(ctx, index, verdict);
  }

  clear_config(ctx) {
    return this.exports.clear_config(ctx);
  }

  declare_name(ctx, buffer, maxSize) {
    return this.exports.declare_name(ctx, buffer, maxSize);
  }

  deinit_louder(ctx) {
    return this.exports.deinit_louder(ctx);
  }

  flush_events(ctx) {
    this.exports.flush_events(ctx);
  }

  force_position(ctx, conversation, actor, targetPosition, fakeChannel, fakePriority) {
    return this.exports.force_position(ctx, conversation, actor, targetPosition, fakeChannel, fakePriority);
  }

  force_warp(ctx, conversation, actor, targetPosition, fakeChannel, fakePriority) {
    return this.exports.force_warp(ctx, conversation, actor, targetPosition, fakeChannel, fakePriority);
  }

  get_actor_additional(ctx, conversation, actor, key) {
    const slot = reserve(this.memory, 8);
    this.exports.get_actor_additional(slot, ctx, conversation, actor, key);
    return take(this.memory, slot, 8);
  }

  get_actor_channel(ctx, conversation, actor) {
    return this.exports.get_actor_channel(ctx, conversation, actor);
  }

  get_actor_flags(ctx, conversation, actor) {
    return this.exports.get_actor_flags(ctx, conversation, actor);
  }

  get_actor_position(ctx, conversation, actor) {
    return this.exports.get_actor_position(ctx, conversation, actor);
  }

  get_actor_priority(ctx, conversation, actor) {
    return this.exports.get_actor_priority(ctx, conversation, actor);
  }

  get_ask(ctx, index) {
    const slot = reserve(this.memory, 36);
    this.exports.get_ask(slot, ctx, index);
    return take(this.memory, slot, 36);
  }

  get_dialogue_rules(ctx) {
    return this.exports.get_dialogue_rules(ctx);
  }

  get_event(ctx, index) {
    const slot = reserve(this.memory, 36);
    this.exports.get_event(slot, ctx, index);
    return take(this.memory, slot, 36);
  }

  get_hub_additional(ctx, conversation, position, key) {
    const slot = reserve(this.memory, 8);
    this.exports.get_hub_additional(slot, ctx, conversation, position, key);
    return take(this.memory, slot, 8);
  }

  get_hub_callbacks(ctx, conversation, position) {
    return this.exports.get_hub_callbacks(ctx, conversation, position);
  }

  get_hub_channel(ctx, conversation, position) {
    return this.exports.get_hub_channel(ctx, conversation, position);
  }

  get_hub_choice_count(ctx, conversation, position, actorPerspective) {
    return this.exports.get_hub_choice_count(ctx, conversation, position, actorPerspective);
  }

  get_hub_choice_link(ctx, conversation, position, actorPerspective, choiceIndex) {
    return this.exports.get_hub_choice_link(ctx, conversation, position, actorPerspective, choiceIndex);
  }

  get_hub_choice_text(ctx, conversation, position, actorPerspective, choiceIndex) {
    return this.exports.get_hub_choice_text(ctx, conversation, position, actorPerspective, choiceIndex);
  }

  get_hub_expressions(ctx, conversation, position) {
    return this.exports.get_hub_expressions(ctx, conversation, position);
  }

  get_hub_flags(ctx, conversation, position) {
    return this.exports.get_hub_flags(ctx, conversation, position);
  }

  get_hub_priority(ctx, conversation, position) {
    return this.exports.get_hub_priority(ctx, conversation, position);
  }

  get_hub_speaker(ctx, conversation, position) {
    return this.exports.get_hub_speaker(ctx, conversation, position);
  }

  get_hub_text(ctx, conversation, position) {
    return this.exports.get_hub_text(ctx, conversation, position);
  }

  get_named_id(ctx, buffer, maxSize) {
    return this.exports.get_named_id(ctx, buffer, maxSize);
  }

  get_next_speaker_listen_to(ctx, speaker, start) {
    return this.exports.get_next_speaker_listen_to(ctx, speaker, start);
  }

  get_speaker_additional(ctx, speaker, key) {
    const slot = reserve(this.memory, 8);
    this.exports.get_speaker_additional(slot, ctx, speaker, key);
    return take(this.memory, slot, 8);
  }

  get_speaker_flags(ctx, speaker) {
    return this.exports.get_speaker_flags(ctx, speaker);
  }

  get_story_state(ctx, conversation, state) {
    const slot = reserve(this.memory, 8);
    this.exports.get_story_state(slot, ctx, conversation, state);
    return take(this.memory, slot, 8);
  }

  init_louder() {
    return this.exports.init_louder();
  }

  load_conversation_config(ctx, buffer, maxSize) {
    return this.exports.load_conversation_config(ctx, buffer, maxSize);
  }

  load_conversation_json(ctx, conversation, buffer, maxSize) {
    return this.exports.load_conversation_json(ctx, conversation, buffer, maxSize);
  }

  make_event_data_actor_added() {
    const slot = reserve(this.memory, 20);
    this.exports.make_event_data_actor_added(slot);
    return take(this.memory, slot, 20);
  }

  make_event_data_actor_removed() {
    const slot = reserve(this.memory, 20);
    this.exports.make_event_data_actor_removed(slot);
    return take(this.memory, slot, 20);
  }

  make_event_data_additional_data_change(change) {
    const slot = reserve(this.memory, 20);
    let view;
    const changeAt = reserve(this.memory, 4);
    view = this.bytes();
    writeAdditionalDataChange(view, changeAt, change);
    this.exports.make_event_data_additional_data_change(slot, changeAt);
    return take(this.memory, slot, 20);
  }

  make_event_data_callback(callback) {
    const slot = reserve(this.memory, 20);
    this.exports.make_event_data_callback(slot, callback);
    return take(this.memory, slot, 20);
  }

  make_event_data_channel_change() {
    const slot = reserve(this.memory, 20);
    this.exports.make_event_data_channel_change(slot);
    return take(this.memory, slot, 20);
  }

  make_event_data_choice_appearance(appearance) {
    const slot = reserve(this.memory, 20);
    let view;
    const appearanceAt = reserve(this.memory, 8);
    view = this.bytes();
    writeChoiceAppearance(view, appearanceAt, appearance);
    this.exports.make_event_data_choice_appearance(slot, appearanceAt);
    return take(this.memory, slot, 20);
  }

  make_event_data_condition(outcome) {
    const slot = reserve(this.memory, 20);
    let view;
    const outcomeAt = reserve(this.memory, 12);
    view = this.bytes();
    writeConditionOutcome(view, outcomeAt, outcome);
    this.exports.make_event_data_condition(slot, outcomeAt);
    return take(this.memory, slot, 20);
  }

  make_event_data_conversation_added(conversation) {
    const slot = reserve(this.memory, 20);
    this.exports.make_event_data_conversation_added(slot, conversation);
    return take(this.memory, slot, 20);
  }

  make_event_data_conversation_removed(conversation) {
    const slot = reserve(this.memory, 20);
    this.exports.make_event_data_conversation_removed(slot, conversation);
    return take(this.memory, slot, 20);
  }

  make_event_data_force_motion(motion) {
    const slot = reserve(this.memory, 20);
    let view;
    const motionAt = reserve(this.memory, 16);
    view = this.bytes();
    writeMotion(view, motionAt, motion);
    this.exports.make_event_data_force_motion(slot, motionAt);
    return take(this.memory, slot, 20);
  }

  make_event_data_load_error(failure) {
    const slot = reserve(this.memory, 20);
    let view;
    const failureAt = reserve(this.memory, 8);
    view = this.bytes();
    writeLoadFailure(view, failureAt, failure);
    this.exports.make_event_data_load_error(slot, failureAt);
    return take(this.memory, slot, 20);
  }

  make_event_data_none() {
    const slot = reserve(this.memory, 20);
    this.exports.make_event_data_none(slot);
    return take(this.memory, slot, 20);
  }

  make_event_data_regular_motion(motion) {
    const slot = reserve(this.memory, 20);
    let view;
    const motionAt = reserve(this.memory, 16);
    view = this.bytes();
    writeMotion(view, motionAt, motion);
    this.exports.make_event_data_regular_motion(slot, motionAt);
    return take(this.memory, slot, 20);
  }

  make_event_data_speaker_added(speaker) {
    const slot = reserve(this.memory, 20);
    this.exports.make_event_data_speaker_added(slot, speaker);
    return take(this.memory, slot, 20);
  }

  make_event_data_speaker_change(speaker) {
    const slot = reserve(this.memory, 20);
    this.exports.make_event_data_speaker_change(slot, speaker);
    return take(this.memory, slot, 20);
  }

  make_event_data_speaker_removed(speaker) {
    const slot = reserve(this.memory, 20);
    this.exports.make_event_data_speaker_removed(slot, speaker);
    return take(this.memory, slot, 20);
  }

  make_event_data_statement(expression) {
    const slot = reserve(this.memory, 20);
    this.exports.make_event_data_statement(slot, expression);
    return take(this.memory, slot, 20);
  }

  make_event_data_text_change(text) {
    const slot = reserve(this.memory, 20);
    this.exports.make_event_data_text_change(slot, text);
    return take(this.memory, slot, 20);
  }

  make_event_data_warp(motion) {
    const slot = reserve(this.memory, 20);
    let view;
    const motionAt = reserve(this.memory, 16);
    view = this.bytes();
    writeMotion(view, motionAt, motion);
    this.exports.make_event_data_warp(slot, motionAt);
    return take(this.memory, slot, 20);
  }

  make_some_ask(value) {
    const slot = reserve(this.memory, 36);
    let view;
    const valueAt = reserve(this.memory, 32);
    view = this.bytes();
    writeLouderAsk(view, valueAt, value);
    this.exports.make_some_ask(slot, valueAt);
    return take(this.memory, slot, 36);
  }

  make_some_event(value) {
    const slot = reserve(this.memory, 36);
    let view;
    const valueAt = reserve(this.memory, 32);
    view = this.bytes();
    writeLouderEvent(view, valueAt, value);
    this.exports.make_some_event(slot, valueAt);
    return take(this.memory, slot, 36);
  }

  make_variant_bool(value) {
    const slot = reserve(this.memory, 8);
    this.exports.make_variant_bool(slot, value);
    return take(this.memory, slot, 8);
  }

  make_variant_float(value) {
    const slot = reserve(this.memory, 8);
    this.exports.make_variant_float(slot, value);
    return take(this.memory, slot, 8);
  }

  make_variant_int(value) {
    const slot = reserve(this.memory, 8);
    this.exports.make_variant_int(slot, value);
    return take(this.memory, slot, 8);
  }

  make_variant_string(value) {
    const slot = reserve(this.memory, 8);
    this.exports.make_variant_string(slot, value);
    return take(this.memory, slot, 8);
  }

  make_variant_zero() {
    const slot = reserve(this.memory, 8);
    this.exports.make_variant_zero(slot);
    return take(this.memory, slot, 8);
  }

  next_actor_from_flags(ctx, conversation, include, exact, start) {
    let view;
    const includeAt = reserve(this.memory, 4);
    view = this.bytes();
    writeActorFlags(view, includeAt, include.bits);
    return this.exports.next_actor_from_flags(ctx, conversation, includeAt, exact, start);
  }

  next_actor_in_channel(ctx, conversation, channel, start) {
    return this.exports.next_actor_in_channel(ctx, conversation, channel, start);
  }

  next_actor_in_channel_and_position(ctx, conversation, channel, position, start) {
    return this.exports.next_actor_in_channel_and_position(ctx, conversation, channel, position, start);
  }

  next_actor_listening(ctx, conversation, actor, start) {
    return this.exports.next_actor_listening(ctx, conversation, actor, start);
  }

  next_actor_listening_base(ctx, conversation, actor, start) {
    return this.exports.next_actor_listening_base(ctx, conversation, actor, start);
  }

  next_actor_listening_to(ctx, conversation, speaker, start) {
    return this.exports.next_actor_listening_to(ctx, conversation, speaker, start);
  }

  next_actor_speak_as(ctx, conversation, actor, start) {
    return this.exports.next_actor_speak_as(ctx, conversation, actor, start);
  }

  next_actor_speaking_as(ctx, conversation, speaker, start) {
    return this.exports.next_actor_speaking_as(ctx, conversation, speaker, start);
  }

  next_hub_by_channel(ctx, conversation, channel, start) {
    return this.exports.next_hub_by_channel(ctx, conversation, channel, start);
  }

  next_hub_by_priority(ctx, conversation, priorityMin, priorityMax, start) {
    return this.exports.next_hub_by_priority(ctx, conversation, priorityMin, priorityMax, start);
  }

  next_hub_from_flags(ctx, conversation, include, exact, start) {
    let view;
    const includeAt = reserve(this.memory, 4);
    view = this.bytes();
    writeHubFlags(view, includeAt, include.bits);
    return this.exports.next_hub_from_flags(ctx, conversation, includeAt, exact, start);
  }

  next_hub_from_tags(ctx, conversation, include, includeCount, exact, start) {
    return this.exports.next_hub_from_tags(ctx, conversation, include, includeCount, exact, start);
  }

  next_hub_heard_by(ctx, conversation, speaker, start) {
    return this.exports.next_hub_heard_by(ctx, conversation, speaker, start);
  }

  next_hub_recipient(ctx, conversation, position, start) {
    return this.exports.next_hub_recipient(ctx, conversation, position, start);
  }

  next_hub_spoken_by(ctx, conversation, speaker, start) {
    return this.exports.next_hub_spoken_by(ctx, conversation, speaker, start);
  }

  next_hub_tag(ctx, conversation, position, start) {
    return this.exports.next_hub_tag(ctx, conversation, position, start);
  }

  push_ask(ctx, ask) {
    return this.exports.push_ask(ctx, ask);
  }

  read_name(ctx, name, buffer, maxSize) {
    return this.exports.read_name(ctx, name, buffer, maxSize);
  }

  read_snapshot_json(ctx, conversation, buffer, maxSize) {
    return this.exports.read_snapshot_json(ctx, conversation, buffer, maxSize);
  }

  read_state_json(ctx, conversation, buffer, maxSize) {
    return this.exports.read_state_json(ctx, conversation, buffer, maxSize);
  }

  remove_actor(ctx, conversation, actor) {
    return this.exports.remove_actor(ctx, conversation, actor);
  }

  remove_conversation(ctx, conversation) {
    return this.exports.remove_conversation(ctx, conversation);
  }

  select_choice(ctx, conversation, actor, choiceIndex) {
    return this.exports.select_choice(ctx, conversation, actor, choiceIndex);
  }

  set_dialogue_rules(ctx, rules) {
    let view;
    const rulesAt = reserve(this.memory, 4);
    view = this.bytes();
    writeDialogueRules(view, rulesAt, rules.bits);
    return this.exports.set_dialogue_rules(ctx, rulesAt);
  }

  set_story_state(ctx, conversation, key, value) {
    let view;
    const valueAt = reserve(this.memory, 8);
    view = this.bytes();
    writeLouderVariant(view, valueAt, value);
    this.exports.set_story_state(ctx, conversation, key, valueAt);
  }

  test_linkage() {
    return this.exports.test_linkage();
  }

  test_push_event(ctx, event) {
    return this.exports.test_push_event(ctx, event);
  }

}

/** Instantiate an engine module and hand back its raw layer. */
export async function instantiate(source) {
  const { instance } = await WebAssembly.instantiate(source, {});
  return new Raw(instance);
}

/** Load a `.wasm` file, an `ArrayBuffer`, or anything `fetch` returns. */
export async function load(source) {
  if (typeof source === 'string' || source instanceof URL) {
    return instantiate(await (await fetch(source)).arrayBuffer());
  }
  return instantiate(source);
}
