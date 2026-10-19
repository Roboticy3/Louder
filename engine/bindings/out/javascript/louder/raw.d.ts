// Bindings for JavaScript, generated from the C header of the Louder engine.
//
// Do not edit. Run `python engine/bindings/generate.py` and this file comes back.

/** How much of linear memory each declared type occupies. */
export declare const SIZE: Readonly<Record<string, number>>;

/** Where each field sits inside its type. */
export declare const OFFSET: Readonly<Record<string, Readonly<Record<string, number>>>>;

/** Tag value to arm name, for every tagged union. */
export declare const KINDS: Readonly<Record<string, Readonly<Record<number, string>>>>;

/** The constants the header declares. */
export declare const CONSTANTS: Readonly<Record<string, number>>;

/** Why a document was refused. Zero means it was not refused. */
export declare const LoadError: Readonly<Record<number, string>>;

/** The raw layer: the module's exports, called with its own conventions. */
export declare class Raw {
  constructor(instance: WebAssembly.Instance);
  readonly exports: WebAssembly.Exports;
  readonly memory: WebAssembly.Memory;
  bytes(): DataView;
  add_actor(ctx: number, conversation: number, startPosition: number, flags: ActorFlags, speakAs: number, speakAsCount: number, listenTo: number, listenToCount: number): number;
  answer_ask(ctx: number, index: number, verdict: number): LouderError;
  clear_config(ctx: number): number;
  declare_name(ctx: number, buffer: number, maxSize: number): number;
  deinit_louder(ctx: number): LouderError;
  flush_events(ctx: number): void;
  force_position(ctx: number, conversation: number, actor: number, targetPosition: number, fakeChannel: number, fakePriority: number): number;
  force_warp(ctx: number, conversation: number, actor: number, targetPosition: number, fakeChannel: number, fakePriority: number): number;
  get_actor_additional(ctx: number, conversation: number, actor: number, key: number): Uint8Array;
  get_actor_channel(ctx: number, conversation: number, actor: number): number;
  get_actor_flags(ctx: number, conversation: number, actor: number): ActorFlags;
  get_actor_position(ctx: number, conversation: number, actor: number): number;
  get_actor_priority(ctx: number, conversation: number, actor: number): number;
  get_ask(ctx: number, index: number): Uint8Array;
  get_dialogue_rules(ctx: number): DialogueRules;
  get_event(ctx: number, index: number): Uint8Array;
  get_hub_additional(ctx: number, conversation: number, position: number, key: number): Uint8Array;
  get_hub_callbacks(ctx: number, conversation: number, position: number): number;
  get_hub_channel(ctx: number, conversation: number, position: number): number;
  get_hub_choice_count(ctx: number, conversation: number, position: number, actorPerspective: number): number;
  get_hub_choice_link(ctx: number, conversation: number, position: number, actorPerspective: number, choiceIndex: number): number;
  get_hub_choice_text(ctx: number, conversation: number, position: number, actorPerspective: number, choiceIndex: number): number;
  get_hub_expressions(ctx: number, conversation: number, position: number): number;
  get_hub_flags(ctx: number, conversation: number, position: number): HubFlags;
  get_hub_priority(ctx: number, conversation: number, position: number): number;
  get_hub_speaker(ctx: number, conversation: number, position: number): number;
  get_hub_text(ctx: number, conversation: number, position: number): number;
  get_named_id(ctx: number, buffer: number, maxSize: number): number;
  get_next_speaker_listen_to(ctx: number, speaker: number, start: number): number;
  get_speaker_additional(ctx: number, speaker: number, key: number): Uint8Array;
  get_speaker_flags(ctx: number, speaker: number): SpeakerFlags;
  get_story_state(ctx: number, conversation: number, state: number): Uint8Array;
  init_louder(): number;
  load_conversation_config(ctx: number, buffer: number, maxSize: number): number;
  load_conversation_json(ctx: number, conversation: number, buffer: number, maxSize: number): number;
  make_event_data_actor_added(): Uint8Array;
  make_event_data_actor_removed(): Uint8Array;
  make_event_data_additional_data_change(change: AdditionalDataChange): Uint8Array;
  make_event_data_callback(callback: number): Uint8Array;
  make_event_data_channel_change(): Uint8Array;
  make_event_data_choice_appearance(appearance: ChoiceAppearance): Uint8Array;
  make_event_data_condition(outcome: ConditionOutcome): Uint8Array;
  make_event_data_conversation_added(conversation: number): Uint8Array;
  make_event_data_conversation_removed(conversation: number): Uint8Array;
  make_event_data_force_motion(motion: Motion): Uint8Array;
  make_event_data_load_error(failure: LoadFailure): Uint8Array;
  make_event_data_none(): Uint8Array;
  make_event_data_regular_motion(motion: Motion): Uint8Array;
  make_event_data_speaker_added(speaker: number): Uint8Array;
  make_event_data_speaker_change(speaker: number): Uint8Array;
  make_event_data_speaker_removed(speaker: number): Uint8Array;
  make_event_data_statement(expression: number): Uint8Array;
  make_event_data_text_change(text: number): Uint8Array;
  make_event_data_warp(motion: Motion): Uint8Array;
  make_some_ask(value: LouderAsk): Uint8Array;
  make_some_event(value: LouderEvent): Uint8Array;
  make_variant_bool(value: boolean): Uint8Array;
  make_variant_float(value: number): Uint8Array;
  make_variant_int(value: number): Uint8Array;
  make_variant_string(value: number): Uint8Array;
  make_variant_zero(): Uint8Array;
  next_actor_from_flags(ctx: number, conversation: number, include: ActorFlags, exact: number, start: number): number;
  next_actor_in_channel(ctx: number, conversation: number, channel: number, start: number): number;
  next_actor_in_channel_and_position(ctx: number, conversation: number, channel: number, position: number, start: number): number;
  next_actor_listening(ctx: number, conversation: number, actor: number, start: number): number;
  next_actor_listening_base(ctx: number, conversation: number, actor: number, start: number): number;
  next_actor_listening_to(ctx: number, conversation: number, speaker: number, start: number): number;
  next_actor_speak_as(ctx: number, conversation: number, actor: number, start: number): number;
  next_actor_speaking_as(ctx: number, conversation: number, speaker: number, start: number): number;
  next_hub_by_channel(ctx: number, conversation: number, channel: number, start: number): number;
  next_hub_by_priority(ctx: number, conversation: number, priorityMin: number, priorityMax: number, start: number): number;
  next_hub_from_flags(ctx: number, conversation: number, include: HubFlags, exact: number, start: number): number;
  next_hub_from_tags(ctx: number, conversation: number, include: number, includeCount: number, exact: number, start: number): number;
  next_hub_heard_by(ctx: number, conversation: number, speaker: number, start: number): number;
  next_hub_recipient(ctx: number, conversation: number, position: number, start: number): number;
  next_hub_spoken_by(ctx: number, conversation: number, speaker: number, start: number): number;
  next_hub_tag(ctx: number, conversation: number, position: number, start: number): number;
  push_ask(ctx: number, ask: number): LouderError;
  read_name(ctx: number, name: number, buffer: number, maxSize: number): number;
  read_snapshot_json(ctx: number, conversation: number, buffer: number, maxSize: number): number;
  read_state_json(ctx: number, conversation: number, buffer: number, maxSize: number): number;
  remove_actor(ctx: number, conversation: number, actor: number): number;
  remove_conversation(ctx: number, conversation: number): number;
  select_choice(ctx: number, conversation: number, actor: number, choiceIndex: number): number;
  set_dialogue_rules(ctx: number, rules: DialogueRules): number;
  set_story_state(ctx: number, conversation: number, key: number, value: LouderVariant): void;
  test_linkage(): number;
  test_push_event(ctx: number, event: number): LouderError;
}

export declare function instantiate(source: ArrayBuffer | Response): Promise<Raw>;
export declare function load(source: string | URL | ArrayBuffer | Response): Promise<Raw>;

export declare function readActorFlags(view: DataView, at: number): unknown;
export declare function readAdditionalDataChange(view: DataView, at: number): unknown;
export declare function readChoiceAppearance(view: DataView, at: number): unknown;
export declare function readLouderVariant_Body(view: DataView, at: number): unknown;
export declare function readConditionOutcome(view: DataView, at: number): unknown;
export declare function readDialogueRules(view: DataView, at: number): unknown;
export declare function readMotion(view: DataView, at: number): unknown;
export declare function readLoadFailure(view: DataView, at: number): unknown;
export declare function readEventData_Body(view: DataView, at: number): unknown;
export declare function readHubFlags(view: DataView, at: number): unknown;
export declare function readLouderAsk(view: DataView, at: number): unknown;
export declare function readLouderEvent(view: DataView, at: number): unknown;
export declare function readLouderOption_LouderAsk_Body(view: DataView, at: number): unknown;
export declare function readLouderOption_LouderEvent_Body(view: DataView, at: number): unknown;
export declare function readSpeakerFlags(view: DataView, at: number): unknown;
export declare function readEventData(view: DataView, at: number): unknown;
export declare function readLouderOption_LouderAsk(view: DataView, at: number): unknown;
export declare function readLouderOption_LouderEvent(view: DataView, at: number): unknown;
export declare function readLouderVariant(view: DataView, at: number): unknown;
