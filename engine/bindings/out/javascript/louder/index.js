// Bindings for JavaScript, generated from the C header of the Louder engine.
//
// Do not edit. Run `python engine/bindings/generate.py` and this file comes back.
/**
 * The Louder engine, from JavaScript.
 *
 * Names are strings, documents are strings, a variant is a number or a
 * string, and an event carries a kind and a decoded body. The engine has no
 * opinion about any of that; this layer is where it is expressed.
 *
 *     const engine = await Louder.load(louderWasm);
 *     engine.loadConfig(document);
 *     engine.loadConversation('chat', document);
 *
 *     const actor = engine.addActor('chat', 0, { speakAs: ['A'] });
 *     engine.choose('chat', actor, 0);
 *     for (const event of engine.drainEvents()) {
 *       console.log(event.kind, event.value);
 *     }
 */

import {
  CONSTANTS,
  KINDS,
  LoadError,
  LouderError,
  Raw,
  TAGS,
  instantiate,
  load,
  readEventData,
  readLouderAsk,
  readLouderVariant,
} from "./raw.js";
import { fromBytes, readInto, reserve, view, withBytes } from "./memory.js";

export { Raw, instantiate, load, KINDS, TAGS, CONSTANTS, LoadError, LouderError };

/** The id of nothing, which is what a failed lookup comes back as. */
export const NAME_NONE = -1;

/** The id of no actor, which is what an actor outside the flow reads as. */
export const ACTOR_NONE = -1;

/** The position of no hub, and where a cursor over hubs starts. */
export const BLANK_POSITION = -1;


function flags(value) {
  return typeof value === "number" ? { bits: value } : (value ?? { bits: 0 });
}

/** One engine context: the name table, the conversations, and the queue. */
export class Louder {
  constructor(raw) {
    this.raw = raw;
    this.ctx = raw.init_louder();
    this.closed = false;
  }

  /** Load an engine module and open a context on it. */
  static async load(source) {
    return new Louder(await load(source));
  }

  static async instantiate(source) {
    return new Louder(await instantiate(source));
  }

  close() {
    if (!this.closed) {
      this.closed = true;
      this.raw.deinit_louder(this.ctx);
    }
  }

  /** The id of some text, or NAME_NONE if it was never declared. */
  lookup(text) {
    const [pointer, length] = withBytes(this.raw.memory, text);
    return this.raw.get_named_id(this.ctx, pointer, length);
  }

  /** The id of some text, declaring it the first time. An id passes through. */
  declare(text) {
    if (typeof text === "number") {
      return text;
    }
    const existing = this.lookup(text);
    if (existing !== NAME_NONE) {
      return existing;
    }
    const [pointer, length] = withBytes(this.raw.memory, text);
    return this.raw.declare_name(this.ctx, pointer, length);
  }

  /** An alias for declare, for when the text really is a name. */
  name(text) {
    return this.declare(text);
  }

  /** The text behind an id, decoded as UTF-8. */
  text(id) {
    return fromBytes(this.#read(this.raw.read_name.bind(this.raw), [this.ctx, Number(id)]));
  }

  /** The text behind an id, or null when the id names nothing. */
  optionalText(id) {
    return Number(id) === NAME_NONE ? null : this.text(id);
  }

  /** Load a conversation from a JSON document. */
  loadConversation(conversation, document) {
    const name = this.declare(conversation);
    const [pointer, length] = withBytes(this.raw.memory, document);
    return LoadError[this.raw.load_conversation_json(this.ctx, name, pointer, length)];
  }

  /** Load the speaker config, which is a singleton. */
  loadConfig(document) {
    const [pointer, length] = withBytes(this.raw.memory, document);
    return LoadError[this.raw.load_conversation_config(this.ctx, pointer, length)];
  }

  /** Remove every speaker. Conversations are left alone. */
  clearConfig() {
    return this.raw.clear_config(this.ctx) === 0;
  }

  /** Remove a conversation and the actors in it. */
  removeConversation(conversation) {
    return this.raw.remove_conversation(this.ctx, this.declare(conversation)) === 0;
  }

  /** The text of a hub. */
  hubText(conversation, position) {
    return this.text(this.raw.get_hub_text(this.ctx, this.declare(conversation), position));
  }

  /** The speaker of a hub. */
  hubSpeaker(conversation, position) {
    return this.text(this.raw.get_hub_speaker(this.ctx, this.declare(conversation), position));
  }

  /** The channel of a hub. */
  hubChannel(conversation, position) {
    return this.text(this.raw.get_hub_channel(this.ctx, this.declare(conversation), position));
  }

  /** The expressions of a hub, which the schema keeps newline separated. */
  hubExpressions(conversation, position) {
    return this.text(this.raw.get_hub_expressions(this.ctx, this.declare(conversation), position));
  }

  /** The callbacks of a hub. */
  hubCallbacks(conversation, position) {
    return this.text(this.raw.get_hub_callbacks(this.ctx, this.declare(conversation), position));
  }

  /** The priority of a hub. */
  hubPriority(conversation, position) {
    return this.raw.get_hub_priority(this.ctx, this.declare(conversation), position);
  }

  /** The flags on a hub, as bits. */
  hubFlags(conversation, position) {
    return this.raw.get_hub_flags(this.ctx, this.declare(conversation), position);
  }

  /** Additional data on a hub, under a key. */
  hubAdditional(conversation, position, key) {
    const bytes = this.raw.get_hub_additional(this.ctx, this.declare(conversation), position, this.declare(key));
    return this.#variant(bytes);
  }

  /** How many choices are available to an actor on a hub. */
  hubChoiceCount(conversation, position, actor) {
    return this.raw.get_hub_choice_count(this.ctx, this.declare(conversation), position, actor);
  }

  /** The text of one choice, which is the text of the hub it leads to. */
  hubChoiceText(conversation, position, actor, index) {
    return this.text(this.raw.get_hub_choice_text(this.ctx, this.declare(conversation), position, actor, index));
  }

  /** Where one choice leads. */
  hubChoiceLink(conversation, position, actor, index) {
    return this.raw.get_hub_choice_link(this.ctx, this.declare(conversation), position, actor, index);
  }

  /** Put an actor into a conversation and return its id. */
  addActor(conversation, position, options = {}) {
    const speakAs = this.#nameList(options.speakAs);
    const listenTo = this.#nameList(options.listenTo);
    return this.raw.add_actor(
      this.ctx,
      this.declare(conversation),
      position,
      flags(options.flags),
      speakAs.pointer,
      speakAs.count,
      listenTo.pointer,
      listenTo.count,
    );
  }

  /** Take an actor out of a conversation. */
  removeActor(conversation, actor) {
    return this.raw.remove_actor(this.ctx, this.declare(conversation), actor) === 0;
  }

  /** Where an actor is standing. */
  actorPosition(conversation, actor) {
    return this.raw.get_actor_position(this.ctx, this.declare(conversation), actor);
  }

  /** Which channel an actor is on. */
  actorChannel(conversation, actor) {
    return this.text(this.raw.get_actor_channel(this.ctx, this.declare(conversation), actor));
  }

  /** The flags on an actor, as bits. */
  actorFlags(conversation, actor) {
    return this.raw.get_actor_flags(this.ctx, this.declare(conversation), actor);
  }

  /** The priority on an actor. */
  actorPriority(conversation, actor) {
    return this.raw.get_actor_priority(this.ctx, this.declare(conversation), actor);
  }

  /** Additional data on an actor, under a key. */
  actorAdditional(conversation, actor, key) {
    const bytes = this.raw.get_actor_additional(this.ctx, this.declare(conversation), actor, this.declare(key));
    return this.#variant(bytes);
  }

  /** Take a choice on the hub an actor is standing on. */
  choose(conversation, actor, index) {
    return this.raw.select_choice(this.ctx, this.declare(conversation), actor, index) === 0;
  }

  /** Move an actor as though it had walked there. */
  warp(conversation, actor, target, options = {}) {
    const channel = options.channel === undefined ? NAME_NONE : this.declare(options.channel);
    return this.raw.force_warp(this.ctx, this.declare(conversation), actor, target, channel, options.priority ?? 0) === 0;
  }

  /** Move an actor straight there. */
  teleport(conversation, actor, target, options = {}) {
    const channel = options.channel === undefined ? NAME_NONE : this.declare(options.channel);
    return this.raw.force_position(this.ctx, this.declare(conversation), actor, target, channel, options.priority ?? 0) === 0;
  }

  /** A list of names as the array of ids the engine reads. */
  #nameList(values) {
    const list = values ?? [];
    const pointer = reserve(this.raw.memory, Math.max(list.length, 1) * 4);
    const bytes = view(this.raw.memory);
    for (let index = 0; index < list.length; index += 1) {
      bytes.setInt32(pointer + index * 4, this.declare(list[index]), true);
    }
    return { pointer, count: list.length };
  }

  /** A story state value, or a zero variant if it was never written. */
  get(conversation, key) {
    return this.#variant(this.raw.get_story_state(this.ctx, this.declare(conversation), this.declare(key)));
  }

  /** Write a story state value. */
  set(conversation, key, value) {
    this.raw.set_story_state(
      this.ctx,
      this.declare(conversation),
      this.declare(key),
      this.#makeVariant(value),
    );
  }

  /** The whole story state as a JSON document. */
  stateJson(conversation) {
    return fromBytes(this.#read(this.raw.read_state_json.bind(this.raw), [this.ctx, this.declare(conversation)], 1 << 16));
  }

  /** The story state and every actor, as a JSON document. */
  snapshotJson(conversation) {
    return fromBytes(this.#read(this.raw.read_snapshot_json.bind(this.raw), [this.ctx, this.declare(conversation)], 1 << 16));
  }

  #makeVariant(value) {
    const tag = TAGS.LouderVariant_Tag;
    if (typeof value === "boolean") {
      return { tag: tag.Int, int: value ? 1 : 0 };
    }
    if (typeof value === "number") {
      return Number.isInteger(value)
        ? { tag: tag.Int, int: value }
        : { tag: tag.Float, float: value };
    }
    return { tag: tag.String, string: this.declare(value) };
  }

  #variant(bytes) {
    const variant = readLouderVariant(new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength), 0);
    const kind = KINDS.LouderVariant[variant.tag];
    if (kind === "Int") {
      return variant.int;
    }
    if (kind === "Float") {
      return variant.float;
    }
    return this.text(variant.string);
  }

  /** Call something that fills a buffer, growing until the whole answer fits. */
  #read(call, args, size) {
    return readInto(this.raw.memory, call, args, size);
  }

  /**
   * Every one of these is a cursor in the C header: a `start`, the thing it
   * returns, and a sentinel for the end of the walk. Here they are
   * generators, so the walk reads as a `for...of` and the caller cannot
   * forget to advance.
   */
  *actorFromFlags(conversation, include, exact) {
    let start = ACTOR_NONE;
    for (;;) {
      const found = this.raw.next_actor_from_flags(this.ctx, this.declare(conversation), include, exact, start);
      if (found === ACTOR_NONE) {
        return;
      }
      yield found;
      start = found;
    }
  }

  *actorInChannel(conversation, channel) {
    let start = ACTOR_NONE;
    for (;;) {
      const found = this.raw.next_actor_in_channel(this.ctx, this.declare(conversation), this.declare(channel), start);
      if (found === ACTOR_NONE) {
        return;
      }
      yield found;
      start = found;
    }
  }

  *actorInChannelAndPosition(conversation, channel, position) {
    let start = ACTOR_NONE;
    for (;;) {
      const found = this.raw.next_actor_in_channel_and_position(this.ctx, this.declare(conversation), this.declare(channel), position, start);
      if (found === ACTOR_NONE) {
        return;
      }
      yield found;
      start = found;
    }
  }

  *actorListening(conversation, actor) {
    let start = NAME_NONE;
    for (;;) {
      const found = this.raw.next_actor_listening(this.ctx, this.declare(conversation), actor, start);
      if (found === NAME_NONE) {
        return;
      }
      yield found;
      start = found;
    }
  }

  *actorListeningBase(conversation, actor) {
    let start = NAME_NONE;
    for (;;) {
      const found = this.raw.next_actor_listening_base(this.ctx, this.declare(conversation), actor, start);
      if (found === NAME_NONE) {
        return;
      }
      yield found;
      start = found;
    }
  }

  *actorListeningTo(conversation, speaker) {
    let start = ACTOR_NONE;
    for (;;) {
      const found = this.raw.next_actor_listening_to(this.ctx, this.declare(conversation), this.declare(speaker), start);
      if (found === ACTOR_NONE) {
        return;
      }
      yield found;
      start = found;
    }
  }

  *actorSpeakAs(conversation, actor) {
    let start = NAME_NONE;
    for (;;) {
      const found = this.raw.next_actor_speak_as(this.ctx, this.declare(conversation), actor, start);
      if (found === NAME_NONE) {
        return;
      }
      yield found;
      start = found;
    }
  }

  *actorSpeakingAs(conversation, speaker) {
    let start = ACTOR_NONE;
    for (;;) {
      const found = this.raw.next_actor_speaking_as(this.ctx, this.declare(conversation), this.declare(speaker), start);
      if (found === ACTOR_NONE) {
        return;
      }
      yield found;
      start = found;
    }
  }

  *hubByChannel(conversation, channel) {
    let start = BLANK_POSITION;
    for (;;) {
      const found = this.raw.next_hub_by_channel(this.ctx, this.declare(conversation), this.declare(channel), start);
      if (found === BLANK_POSITION) {
        return;
      }
      yield found;
      start = found;
    }
  }

  *hubByPriority(conversation, priorityMin, priorityMax) {
    let start = BLANK_POSITION;
    for (;;) {
      const found = this.raw.next_hub_by_priority(this.ctx, this.declare(conversation), priorityMin, priorityMax, start);
      if (found === BLANK_POSITION) {
        return;
      }
      yield found;
      start = found;
    }
  }

  *hubFromFlags(conversation, include, exact) {
    let start = BLANK_POSITION;
    for (;;) {
      const found = this.raw.next_hub_from_flags(this.ctx, this.declare(conversation), include, exact, start);
      if (found === BLANK_POSITION) {
        return;
      }
      yield found;
      start = found;
    }
  }

  *hubFromTags(conversation, include, includeCount, exact) {
    let start = BLANK_POSITION;
    for (;;) {
      const found = this.raw.next_hub_from_tags(this.ctx, this.declare(conversation), include, includeCount, exact, start);
      if (found === BLANK_POSITION) {
        return;
      }
      yield found;
      start = found;
    }
  }

  *hubHeardBy(conversation, speaker) {
    let start = BLANK_POSITION;
    for (;;) {
      const found = this.raw.next_hub_heard_by(this.ctx, this.declare(conversation), this.declare(speaker), start);
      if (found === BLANK_POSITION) {
        return;
      }
      yield found;
      start = found;
    }
  }

  *hubRecipient(conversation, position) {
    let start = NAME_NONE;
    for (;;) {
      const found = this.raw.next_hub_recipient(this.ctx, this.declare(conversation), position, start);
      if (found === NAME_NONE) {
        return;
      }
      yield found;
      start = found;
    }
  }

  *hubSpokenBy(conversation, speaker) {
    let start = BLANK_POSITION;
    for (;;) {
      const found = this.raw.next_hub_spoken_by(this.ctx, this.declare(conversation), this.declare(speaker), start);
      if (found === BLANK_POSITION) {
        return;
      }
      yield found;
      start = found;
    }
  }

  *hubTag(conversation, position) {
    let start = NAME_NONE;
    for (;;) {
      const found = this.raw.next_hub_tag(this.ctx, this.declare(conversation), position, start);
      if (found === NAME_NONE) {
        return;
      }
      yield found;
      start = found;
    }
  }

  /** Every queued event, oldest first, leaving the queue alone. */
  *events() {
    for (let index = 0; ; index += 1) {
      const bytes = this.raw.get_event(this.ctx, index);
      const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
      if (view.getInt32(0, true) !== 1) {
        return;
      }
      yield this.#event(view);
    }
  }

  /** Every queued event, oldest first, and empty the queue. */
  drainEvents() {
    const out = [...this.events()];
    this.raw.flush_events(this.ctx);
    return out;
  }

  #event(view) {
    const data = readEventData(view, 16);
    const kind = KINDS.EventData[data.tag] ?? String(data.tag);
    return {
      conversation: this.text(view.getInt32(4, true)),
      channel: this.text(view.getInt32(8, true)),
      actor: view.getInt32(12, true),
      kind,
      value: this.#eventValue(kind, data),
    };
  }

  /** The arm the tag names, already decoded. */
  #eventValue(kind, data) {
    switch (kind) {
      case "None":
        return undefined;
      case "RegularMotion":
        return data.regularMotion;
      case "Warp":
        return data.warp;
      case "ForceMotion":
        return data.forceMotion;
      case "ChoiceAppearance":
        return data.choiceAppearance;
      case "ChannelChange":
        return undefined;
      case "AdditionalDataChange":
        return data.additionalDataChange;
      case "SpeakerChange":
        return data.speakerChange;
      case "TextChange":
        return data.textChange;
      case "Statement":
        return data.statement;
      case "Condition":
        return data.condition;
      case "Callback":
        return data.callback;
      case "ActorAdded":
        return undefined;
      case "ActorRemoved":
        return undefined;
      case "SpeakerAdded":
        return data.speakerAdded;
      case "SpeakerRemoved":
        return data.speakerRemoved;
      case "ConversationAdded":
        return data.conversationAdded;
      case "ConversationRemoved":
        return data.conversationRemoved;
      case "LoadError":
        return data.loadError;
      default:
        return undefined;
    }
  }

  /** Every pending ask. The order is not stable. */
  *asks() {
    for (let index = 0; ; index += 1) {
      const bytes = this.raw.get_ask(this.ctx, index);
      const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
      if (view.getInt32(0, true) !== 1) {
        return;
      }
      const ask = readLouderAsk(view, 4);
      yield {
        conversation: this.text(ask.conversation),
        channel: this.text(ask.channel),
        actor: ask.actor,
        ask: this.text(ask.ask),
      };
    }
  }

  /** Answer the ask at an index. Answering shifts the ones after it. */
  answer(index, verdict) {
    return this.raw.answer_ask(this.ctx, index, verdict) === 0;
  }

  /** Call any entry point by name, with the signatures of the raw layer. */
  call(name, ...args) {
    return this.raw[name](...args);
  }

  /** The dialogue rules in effect. */
  rules() {
    return this.raw.get_dialogue_rules(this.ctx);
  }

  /** Replace the dialogue rules. They take effect on the next move. */
  setRules(rules) {
    return this.raw.set_dialogue_rules(this.ctx, flags(rules)) === 0;
  }
}
