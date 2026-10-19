// Bindings for JavaScript, generated from the C header of the Louder engine.
//
// Do not edit. Run `python engine/bindings/generate.py` and this file comes back.

export { Raw, KINDS, CONSTANTS } from "./raw.js";
import type { Raw } from "./raw.js";

/** One queued event, with the names in it already decoded. */
export interface Event {
  conversation: string;
  channel: string;
  actor: number;
  kind: string;
  value: unknown;
}

/** A question the engine has asked and is waiting on. */
export interface Ask {
  conversation: string;
  channel: string;
  actor: number;
  ask: string;
}

/** Options for putting an actor into a conversation. */
export interface ActorOptions {
  speakAs?: Array<string | number>;
  listenTo?: Array<string | number>;
  flags?: number;
}

/** Options for moving an actor where it could not otherwise reach. */
export interface MoveOptions {
  channel?: string | number;
  priority?: number;
}

export type LoadError = string;

export declare const NAME_NONE: number;
export declare const ACTOR_NONE: number;
export declare const BLANK_POSITION: number;

/** Flag bits describing an actor. */
export interface ActorFlags {
  bits: number;
}

/** A key of an actor's additional data being replaced. */
export interface AdditionalDataChange {
  key: number;
}

/** A choice becoming available on a hub. */
export interface ChoiceAppearance {
  position: number;
  choiceIndex: number;
}

/** What a condition on a hub came to, when an actor landed on it. */
export interface ConditionOutcome {
  condition: number;
  outcome: LouderVariant;
}

/** The dialogue rules a host can turn on and off. Both are on by default, because each is what makes the engine usable without every conversation having to be written around its absence. They are gathered here rather than left as separate switches so that a host reads and writes the whole configuration in one call, and so that adding a rule does not add another pair of engine calls. */
export interface DialogueRules {
  bits: number;
}

/** Flag bits describing a hub. */
export interface HubFlags {
  bits: number;
}

/** A record in a document the engine could not read. Carries the position of the record that was at fault, or LOUDER_POSITION_NONE when the problem was with the document as a whole. */
export interface LoadFailure {
  position: number;
  error: number;
}

/** A question the engine has asked the host and is waiting on. Answering an ask moves it to the back of the cache, so indexes shift; read the cache again after each answer. */
export interface LouderAsk {
  conversation: number;
  channel: number;
  actor: number;
  motion: Motion;
  ask: number;
}

/** One entry in the event queue. The queue is polled from index 0 upwards and emptied with LouderContext::flush_events(super::context::LouderContext). */
export interface LouderEvent {
  conversation: number;
  channel: number;
  actor: number;
  data: EventData;
}

/** One move of an actor from one hub to another. Every event a move produces carries the same motion, which is how a host reads where an actor came from and where it went. */
export interface Motion {
  sourcePosition: number;
  targetPosition: number;
  choiceIndex: number;
  background: boolean;
}

/** Flag bits describing a speaker. */
export interface SpeakerFlags {
  bits: number;
}

/** What an LouderEvent is about. A tagged union cannot be read as one value by a host that has to switch on it, so this is the discriminant of an EventData as an `int`, and the payload is read through the accessors that match it. A host that does not recognise a value is looking at something a newer engine produced. */
export interface EventData {
  tag: number;
  regularMotion?: Motion;
  warp?: Motion;
  forceMotion?: Motion;
  choiceAppearance?: ChoiceAppearance;
  additionalDataChange?: AdditionalDataChange;
  speakerChange?: number;
  textChange?: number;
  statement?: number;
  condition?: ConditionOutcome;
  callback?: number;
  speakerAdded?: number;
  speakerRemoved?: number;
  conversationAdded?: number;
  conversationRemoved?: number;
  loadError?: LoadFailure;
}

/** Custom Option type exposed to the C FFI */
export interface LouderOption_LouderAsk {
  tag: number;
}

/** Custom Option type exposed to the C FFI */
export interface LouderOption_LouderEvent {
  tag: number;
}

/** One unit of data in an actor, hub, speaker, or story state. Booleans are not a separate kind: they are carried as LouderVariant::Int 1 or 0, which is what makes them interoperable with the arithmetic and comparison rules of the engine guide. */
export interface LouderVariant {
  tag: number;
  int?: number;
  float?: number;
  string?: number;
}

/** One engine context: the name table, the conversations, and the queue. */
export declare class Louder {
  constructor(raw: Raw);
  readonly raw: Raw;
  readonly ctx: number;
  readonly closed: boolean;
  close(): void;
  static load(source: string | URL | ArrayBuffer | Response): Promise<Louder>;
  static instantiate(source: ArrayBuffer | Response): Promise<Louder>;
  lookup(text: string): number;
  declare(text: string | number): number;
  name(text: string | number): number;
  text(id: number): string;
  optionalText(id: number): string | null;
  loadConversation(conversation: string, document: string): LoadError;
  loadConfig(document: string): LoadError;
  clearConfig(): boolean;
  removeConversation(conversation: string): boolean;
  hubText(conversation: string, position: number): string;
  hubSpeaker(conversation: string, position: number): string;
  hubChannel(conversation: string, position: number): string;
  hubExpressions(conversation: string, position: number): string;
  hubCallbacks(conversation: string, position: number): string;
  hubPriority(conversation: string, position: number): number;
  hubFlags(conversation: string, position: number): number;
  hubAdditional(conversation: string, position: number, key: string): number | string;
  hubChoiceCount(conversation: string, position: number, actor: number): number;
  hubChoiceText(conversation: string, position: number, actor: number, index: number): string;
  hubChoiceLink(conversation: string, position: number, actor: number, index: number): number;
  addActor(conversation: string, position: number, options?: ActorOptions): number;
  removeActor(conversation: string, actor: number): boolean;
  actorPosition(conversation: string, actor: number): number;
  actorChannel(conversation: string, actor: number): string;
  actorFlags(conversation: string, actor: number): number;
  actorPriority(conversation: string, actor: number): number;
  actorAdditional(conversation: string, actor: number, key: string): number | string;
  choose(conversation: string, actor: number, index: number): boolean;
  warp(conversation: string, actor: number, target: number, options?: MoveOptions): boolean;
  teleport(conversation: string, actor: number, target: number, options?: MoveOptions): boolean;
  get(conversation: string, key: string): number | string;
  set(conversation: string, key: string, value: number | string | boolean): void;
  stateJson(conversation: string): string;
  snapshotJson(conversation: string): string;
  events(): Generator<Event>;
  drainEvents(): Event[];
  asks(): Generator<Ask>;
  answer(index: number, verdict: number): boolean;
  call(name: string, ...args: unknown[]): unknown;
  rules(): number;
  setRules(rules: number): boolean;
  *actorFromFlags(conversation: string | number, include: number, exact: number): Generator<number>;
  *actorInChannel(conversation: string | number, channel: string | number): Generator<number>;
  *actorInChannelAndPosition(conversation: string | number, channel: string | number, position: number): Generator<number>;
  *actorListening(conversation: string | number, actor: number): Generator<number>;
  *actorListeningBase(conversation: string | number, actor: number): Generator<number>;
  *actorListeningTo(conversation: string | number, speaker: string | number): Generator<number>;
  *actorSpeakAs(conversation: string | number, actor: number): Generator<number>;
  *actorSpeakingAs(conversation: string | number, speaker: string | number): Generator<number>;
  *hubByChannel(conversation: string | number, channel: string | number): Generator<number>;
  *hubByPriority(conversation: string | number, priorityMin: number, priorityMax: number): Generator<number>;
  *hubFromFlags(conversation: string | number, include: number, exact: number): Generator<number>;
  *hubFromTags(conversation: string | number, include: number, includeCount: number, exact: number): Generator<number>;
  *hubHeardBy(conversation: string | number, speaker: string | number): Generator<number>;
  *hubRecipient(conversation: string | number, position: number): Generator<number>;
  *hubSpokenBy(conversation: string | number, speaker: string | number): Generator<number>;
  *hubTag(conversation: string | number, position: number): Generator<number>;
}

