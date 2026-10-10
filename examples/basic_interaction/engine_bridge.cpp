#include "engine_bridge.h"

#include <algorithm>

namespace louder_demo {

namespace {

// Enough for any name in these documents, with room to spare. The engine writes
// a terminator, so this is passed as size - 1.
constexpr size_t kNameBuffer = 8192;

// The engine stops reporting choices past this many. It has no "how many choices
// does this hub have in total" call, so the available list is discovered by
// walking indices until one is refused. The bound keeps a mistake in the
// document from turning into an unbounded loop.
constexpr int32_t kMaxChoices = 64;

}  // namespace

Engine::Engine() {
  ctx_ = init_louder();
}

Engine::~Engine() {
  if (ctx_ != nullptr) {
    deinit_louder(ctx_);
    ctx_ = nullptr;
  }
}

NameId Engine::lookup_or_declare(const std::string& text) {
  const uint8_t* bytes = reinterpret_cast<const uint8_t*>(text.data());
  const size_t size = text.size();

  NameId existing = get_named_id(ctx_, bytes, size);
  if (existing != LOUDER_NAME_NONE) {
    return existing;
  }
  return declare_name(ctx_, bytes, size);
}

NameId Engine::name(const std::string& text) {
  for (const auto& entry : name_cache_) {
    if (entry.first == text) {
      return entry.second;
    }
  }
  NameId id = lookup_or_declare(text);
  name_cache_.emplace_back(text, id);
  return id;
}

std::string Engine::text_of(NameId name) {
  if (name == LOUDER_NAME_NONE) {
    return "";
  }
  std::vector<uint8_t> buffer(kNameBuffer, 0);
  size_t written = read_name(ctx_, name, buffer.data(), buffer.size() - 1, LOUDER_ENCODING_UTF8);
  return std::string(reinterpret_cast<const char*>(buffer.data()), written);
}

LoadError Engine::load_conversation(const std::string& conversation_name, const std::string& document) {
  NameId id = name(conversation_name);
  return load_conversation_json(ctx_, id,
                                const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(document.data())),
                                document.size(), LOUDER_ENCODING_UTF8);
}

LoadError Engine::load_config(const std::string& document) {
  return load_conversation_config(ctx_,
                                  const_cast<uint8_t*>(reinterpret_cast<const uint8_t*>(document.data())),
                                  document.size(), LOUDER_ENCODING_UTF8);
}

Position Engine::find_tag(const std::string& tag) {
  // The conversation name is fixed for this demo, so it is interned here rather
  // than threaded through every query.
  NameId conversation = name("jail");
  NameId tag_id = name(tag);
  return next_hub_from_tags(ctx_, conversation, &tag_id, 1, 0, -1);
}

std::string Engine::speaker_of(Position position) {
  return text_of(get_hub_speaker(ctx_, name("jail"), position));
}

std::string Engine::text_at(Position position) {
  return text_of(get_hub_text(ctx_, name("jail"), position));
}

bool Engine::is_auto(Position position) {
  HubFlags flags = get_hub_flags(ctx_, name("jail"), position);
  return (flags.bits & HubFlags_AUTO.bits) != 0;
}

std::vector<Choice> Engine::available_choices(NameId conversation, Position position, ActorId actor) {
  std::vector<Choice> choices;
  for (int32_t index = 0; index < kMaxChoices; ++index) {
    // The engine refuses indices that are out of range *or* that this actor may
    // not take, and reports both as BLANK_POSITION, so a refusal ends the walk.
    int32_t link = get_hub_choice_link(ctx_, conversation, position, actor, index);
    if (link == kNoPosition) {
      break;
    }
    Choice choice;
    choice.general_index = index;
    choice.link = link;
    choice.text = text_of(get_hub_choice_text(ctx_, conversation, position, actor, index));
    choices.push_back(choice);
  }
  return choices;
}

int32_t Engine::choice_count(NameId conversation, Position position, ActorId actor) {
  return get_hub_choice_count(ctx_, conversation, position, actor);
}

int32_t Engine::choice_link(NameId conversation, Position position, ActorId actor, int32_t general_index) {
  return get_hub_choice_link(ctx_, conversation, position, actor, general_index);
}

ActorId Engine::add_actor(NameId conversation, Position start, const std::string& speaker) {
  NameId speaker_id = name(speaker);
  // Qualified, so the engine's C entry point is not mistaken for this member.
  return ::add_actor(ctx_, conversation, start, ActorFlags(), &speaker_id, 1, nullptr, 0);
}

Position Engine::position_of(NameId conversation, ActorId actor) {
  return get_actor_position(ctx_, conversation, actor);
}

std::string Engine::channel_of(NameId conversation, ActorId actor) {
  return text_of(get_actor_channel(ctx_, conversation, actor));
}

bool Engine::select_choice(NameId conversation, ActorId actor, int32_t general_choice_index) {
  return ::select_choice(ctx_, conversation, actor, general_choice_index) == 0;
}

std::vector<LouderEvent> Engine::drain_events() {
  std::vector<LouderEvent> events;
  for (size_t index = 0;; ++index) {
    LouderOption<LouderEvent> option = get_event(ctx_, index);
    if (option.tag == LouderOption<LouderEvent>::Tag::None) {
      break;
    }
    events.push_back(option.some._0);
  }
  flush_events(ctx_);
  return events;
}

}  // namespace louder_demo
