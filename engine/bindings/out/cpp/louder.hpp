// Bindings for C++, generated from the C header of the Louder engine.
//
// Do not edit. Run `python engine/bindings/generate.py` and this file comes back.
//
// The C++ layer over `louder.h`. A host adds this one file to its build.

#ifndef LOUDER_HPP
#define LOUDER_HPP

#include <cstddef>
#include <cstdint>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

extern "C" {
#include "louder.h"
}

namespace louder {

// The boundary, in shorter names.
using Name = LouderName;
using Position = LouderPosition;
using Actor = LouderActor;
using Callback = LouderCallback;
using Expression = LouderExpression;
using LoadErrorCode = LoadError;

// Why a document was refused.
enum class Failure : LoadErrorCode {
    OK = LOUDER_LOAD_OK,
    MALFORMED_JSON = LOUDER_LOAD_MALFORMED_JSON,
    NOT_A_LIST = LOUDER_LOAD_NOT_A_LIST,
    NOT_AN_OBJECT = LOUDER_LOAD_NOT_AN_OBJECT,
    ALREADY_LOADED = LOUDER_LOAD_ALREADY_LOADED,
};

/// A document load that was not accepted, with the reason the engine gave.
class LoadFailure : public std::runtime_error {
public:
    explicit LoadFailure(Failure reason)
        : std::runtime_error("the engine refused the document"), reason_(reason) {}
    [[nodiscard]] Failure reason() const noexcept { return reason_; }

private:
    Failure reason_;
};

/// One engine context.
///
/// Owns the name table, every conversation, and the event queue. The
/// destructor closes it, which is the whole point of the class existing.
class Louder {
public:
    Louder() : ctx_(init_louder()) {
        if (ctx_ == nullptr) {
            throw std::runtime_error("the engine did not start");
        }
    }
    ~Louder() { close(); }

    Louder(const Louder &) = delete;
    Louder &operator=(const Louder &) = delete;

    Louder(Louder &&moved) noexcept : ctx_(moved.ctx_) { moved.ctx_ = nullptr; }
    Louder &operator=(Louder &&moved) noexcept {
        if (this != &moved) {
            close();
            ctx_ = moved.ctx_;
            moved.ctx_ = nullptr;
        }
        return *this;
    }

    /// Release the context and everything it holds.
    void close() noexcept {
        if (ctx_ != nullptr) {
            deinit_louder(ctx_);
            ctx_ = nullptr;
        }
    }

    /// The raw context pointer, for anything this layer does not wrap.
    [[nodiscard]] LouderContext *handle() const noexcept { return ctx_; }

    // --- names ---

    /// The id of some text, or LOUDER_NAME_NONE if it was never declared.
    [[nodiscard]] Name lookup(std::string_view text) const {
        return get_named_id(ctx_,
                            reinterpret_cast<const uint8_t *>(text.data()),
                            text.size());
    }

    /// The id of some text, declaring it the first time.
    ///
    /// `declare_name` refuses to make a second id for text that already has
    /// one, returning LOUDER_NAME_NONE, so this looks first and declares
    /// second. The engine splits them on purpose; a host should not have to
    /// know.
    [[nodiscard]] Name name(std::string_view text) const {
        const Name found = lookup(text);
        if (found != LOUDER_NAME_NONE) {
            return found;
        }
        return declare_name(ctx_,
                            reinterpret_cast<const uint8_t *>(text.data()),
                            text.size());
    }

    /// The text behind an id.
    ///
    /// The engine reports how many bytes fitted rather than refusing, so the
    /// only way to be sure of the whole answer is to ask again with more
    /// room. Two calls is the ordinary case.
    [[nodiscard]] std::string text(Name id) const {
        if (id == LOUDER_NAME_NONE) {
            return {};
        }
        for (std::size_t size = 4096;; size *= 4) {
            std::string buffer(size, '\0');
            const std::size_t got = read_name(ctx_, id,
                                              reinterpret_cast<uint8_t *>(buffer.data()),
                                              size - 1);
            if (got == 0) {
                return {};
            }
            if (got + 1 < size) {
                buffer.resize(got);
                return buffer;
            }
        }
    }

    // --- documents ---

    /// Load a conversation from a JSON document.
    void load_conversation(Name conversation, std::string_view document) {
        const LoadErrorCode code = load_conversation_json(
            ctx_, conversation,
            reinterpret_cast<uint8_t *>(const_cast<char *>(document.data())),
            document.size());
        raise(code);
    }

    /// Load the speaker config, which is a singleton.
    void load_config(std::string_view document) {
        const LoadErrorCode code = load_conversation_config(
            ctx_,
            reinterpret_cast<uint8_t *>(const_cast<char *>(document.data())),
            document.size());
        raise(code);
    }

    /// Throw when a load was not accepted, with the engine's reason.
    static void raise(LoadErrorCode code) {
        if (code != LOUDER_LOAD_OK) {
            throw LoadFailure(static_cast<Failure>(code));
        }
    }

    // --- the JSON reads ---

    /// The whole story state as a JSON document.
    [[nodiscard]] std::string state_json(Name conversation) const {
        return read_growing(
            [&](uint8_t *at, std::size_t size) {
                return read_state_json(ctx_, conversation, at, size);
            });
    }

    /// The story state and every actor, as a JSON document.
    [[nodiscard]] std::string snapshot_json(Name conversation) const {
        return read_growing(
            [&](uint8_t *at, std::size_t size) {
                return read_snapshot_json(ctx_, conversation, at, size);
            });
    }

    // --- story state ---

    /// A story state value, as whatever it carries.
    [[nodiscard]] int get_int(Name conversation, std::string_view key) const {
        const LouderVariant value = get_story_state(ctx_, conversation, name(key));
        return value.tag == Int ? value.body.int_ : 0;
    }

    [[nodiscard]] float get_float(Name conversation, std::string_view key) const {
        const LouderVariant value = get_story_state(ctx_, conversation, name(key));
        return value.tag == Float ? value.body.float_ : 0.0f;
    }

    [[nodiscard]] std::string get_text(Name conversation, std::string_view key) const {
        const LouderVariant value = get_story_state(ctx_, conversation, name(key));
        return value.tag == String ? text(value.body.string) : std::string{};
    }

    /// Write an integer story state value.
    void set_int(Name conversation, std::string_view key, int value) {
        set_story_state(ctx_, conversation, name(key), make_variant_int(value));
    }

    /// Write a float story state value.
    void set_float(Name conversation, std::string_view key, float value) {
        set_story_state(ctx_, conversation, name(key), make_variant_float(value));
    }

    /// Write a text story state value, stored as the interned name.
    void set_text(Name conversation, std::string_view key, std::string_view value) {
        set_story_state(ctx_, conversation, name(key),
                        make_variant_string(name(value)));
    }

private:
    /// Read into a buffer that grows until the whole answer fits.
    template <typename F>
    static std::string read_growing(F &&call) {
        for (std::size_t size = 4096;; size *= 4) {
            std::string buffer(size, '\0');
            const auto written = call(reinterpret_cast<uint8_t *>(buffer.data()), size - 1);
            if (written < 0) {
                throw std::runtime_error("the engine refused the read");
            }
            if (static_cast<std::size_t>(written) + 1 < size) {
                buffer.resize(static_cast<std::size_t>(written));
                return buffer;
            }
        }
    }

    LouderContext *ctx_;
};

/// Iterate the speakers that `speaker` is listening to.
///
/// Reads as `for (const auto hub : louder::SpeakerListenTo(...))` rather than
/// as the loop it wraps.
class SpeakerListenTo {
public:
    SpeakerListenTo(const LouderContext* ctx, int32_t speaker, int32_t start, int32_t end)
        : ctx_(ctx), speaker_(speaker),
          start_(start), end_(end) {}

    class Iterator {
    public:
        Iterator(const SpeakerListenTo &owner, int32_t current)
            : owner_(owner), current_(current) {}

        bool operator==(const Iterator &other) const { return current_ == other.current_; }
        int32_t operator*() const { return current_; }
        Iterator &operator++() {
            current_ = get_next_speaker_listen_to(owner_.ctx_, owner_.speaker_, current_);
            return *this;
        }

    private:
        const SpeakerListenTo &owner_;
        int32_t current_;
    };

    [[nodiscard]] Iterator begin() const {
        return Iterator(*this, get_next_speaker_listen_to(ctx_, speaker_, start_));
    }
    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }

    /// The engine ends this walk with the value its documentation calls
    /// BLANK_POSITION, which the header does not declare.
private:
    const LouderContext* ctx_;
    int32_t speaker_;
    int32_t start_;
    int32_t end_;
};

/// Iterate the actors matching certain flags.
///
/// Reads as `for (const auto hub : louder::ActorFromFlags(...))` rather than
/// as the loop it wraps.
class ActorFromFlags {
public:
    ActorFromFlags(const LouderContext* ctx, int32_t conversation, ActorFlags include, int32_t exact, int32_t start, int32_t end)
        : ctx_(ctx), conversation_(conversation), include_(include), exact_(exact),
          start_(start), end_(end) {}

    class Iterator {
    public:
        Iterator(const ActorFromFlags &owner, int32_t current)
            : owner_(owner), current_(current) {}

        bool operator==(const Iterator &other) const { return current_ == other.current_; }
        int32_t operator*() const { return current_; }
        Iterator &operator++() {
            current_ = next_actor_from_flags(owner_.ctx_, owner_.conversation_, owner_.include_, owner_.exact_, current_);
            return *this;
        }

    private:
        const ActorFromFlags &owner_;
        int32_t current_;
    };

    [[nodiscard]] Iterator begin() const {
        return Iterator(*this, next_actor_from_flags(ctx_, conversation_, include_, exact_, start_));
    }
    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }

    /// The engine ends this walk with the value its documentation calls
    /// BLANK_POSITION, which the header does not declare.
private:
    const LouderContext* ctx_;
    int32_t conversation_;
    ActorFlags include_;
    int32_t exact_;
    int32_t start_;
    int32_t end_;
};

/// Iterate the actors in a channel.
///
/// Reads as `for (const auto hub : louder::ActorInChannel(...))` rather than
/// as the loop it wraps.
class ActorInChannel {
public:
    ActorInChannel(const LouderContext* ctx, int32_t conversation, int32_t channel, int32_t start, int32_t end)
        : ctx_(ctx), conversation_(conversation), channel_(channel),
          start_(start), end_(end) {}

    class Iterator {
    public:
        Iterator(const ActorInChannel &owner, int32_t current)
            : owner_(owner), current_(current) {}

        bool operator==(const Iterator &other) const { return current_ == other.current_; }
        int32_t operator*() const { return current_; }
        Iterator &operator++() {
            current_ = next_actor_in_channel(owner_.ctx_, owner_.conversation_, owner_.channel_, current_);
            return *this;
        }

    private:
        const ActorInChannel &owner_;
        int32_t current_;
    };

    [[nodiscard]] Iterator begin() const {
        return Iterator(*this, next_actor_in_channel(ctx_, conversation_, channel_, start_));
    }
    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }

    /// The engine ends this walk with the value its documentation calls
    /// BLANK_POSITION, which the header does not declare.
private:
    const LouderContext* ctx_;
    int32_t conversation_;
    int32_t channel_;
    int32_t start_;
    int32_t end_;
};

/// Iterate the actors in a channel at a specific position.
///
/// Reads as `for (const auto hub : louder::ActorInChannelAndPosition(...))` rather than
/// as the loop it wraps.
class ActorInChannelAndPosition {
public:
    ActorInChannelAndPosition(const LouderContext* ctx, int32_t conversation, int32_t channel, int32_t position, int32_t start, int32_t end)
        : ctx_(ctx), conversation_(conversation), channel_(channel), position_(position),
          start_(start), end_(end) {}

    class Iterator {
    public:
        Iterator(const ActorInChannelAndPosition &owner, int32_t current)
            : owner_(owner), current_(current) {}

        bool operator==(const Iterator &other) const { return current_ == other.current_; }
        int32_t operator*() const { return current_; }
        Iterator &operator++() {
            current_ = next_actor_in_channel_and_position(owner_.ctx_, owner_.conversation_, owner_.channel_, owner_.position_, current_);
            return *this;
        }

    private:
        const ActorInChannelAndPosition &owner_;
        int32_t current_;
    };

    [[nodiscard]] Iterator begin() const {
        return Iterator(*this, next_actor_in_channel_and_position(ctx_, conversation_, channel_, position_, start_));
    }
    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }

    /// The engine ends this walk with the value its documentation calls
    /// BLANK_POSITION, which the header does not declare.
private:
    const LouderContext* ctx_;
    int32_t conversation_;
    int32_t channel_;
    int32_t position_;
    int32_t start_;
    int32_t end_;
};

/// Iterate the speakers an actor listens to.
///
/// Reads as `for (const auto hub : louder::ActorListening(...))` rather than
/// as the loop it wraps.
class ActorListening {
public:
    ActorListening(const LouderContext* ctx, int32_t conversation, int32_t actor, int32_t start, int32_t end)
        : ctx_(ctx), conversation_(conversation), actor_(actor),
          start_(start), end_(end) {}

    class Iterator {
    public:
        Iterator(const ActorListening &owner, int32_t current)
            : owner_(owner), current_(current) {}

        bool operator==(const Iterator &other) const { return current_ == other.current_; }
        int32_t operator*() const { return current_; }
        Iterator &operator++() {
            current_ = next_actor_listening(owner_.ctx_, owner_.conversation_, owner_.actor_, current_);
            return *this;
        }

    private:
        const ActorListening &owner_;
        int32_t current_;
    };

    [[nodiscard]] Iterator begin() const {
        return Iterator(*this, next_actor_listening(ctx_, conversation_, actor_, start_));
    }
    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }

    /// The engine ends this walk with the value its documentation calls
    /// BLANK_POSITION, which the header does not declare.
private:
    const LouderContext* ctx_;
    int32_t conversation_;
    int32_t actor_;
    int32_t start_;
    int32_t end_;
};

/// Iterate the base speakers an actor listens to.
///
/// Reads as `for (const auto hub : louder::ActorListeningBase(...))` rather than
/// as the loop it wraps.
class ActorListeningBase {
public:
    ActorListeningBase(const LouderContext* ctx, int32_t conversation, int32_t actor, int32_t start, int32_t end)
        : ctx_(ctx), conversation_(conversation), actor_(actor),
          start_(start), end_(end) {}

    class Iterator {
    public:
        Iterator(const ActorListeningBase &owner, int32_t current)
            : owner_(owner), current_(current) {}

        bool operator==(const Iterator &other) const { return current_ == other.current_; }
        int32_t operator*() const { return current_; }
        Iterator &operator++() {
            current_ = next_actor_listening_base(owner_.ctx_, owner_.conversation_, owner_.actor_, current_);
            return *this;
        }

    private:
        const ActorListeningBase &owner_;
        int32_t current_;
    };

    [[nodiscard]] Iterator begin() const {
        return Iterator(*this, next_actor_listening_base(ctx_, conversation_, actor_, start_));
    }
    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }

    /// The engine ends this walk with the value its documentation calls
    /// BLANK_POSITION, which the header does not declare.
private:
    const LouderContext* ctx_;
    int32_t conversation_;
    int32_t actor_;
    int32_t start_;
    int32_t end_;
};

/// Iterate the actors listening to a speaker.
///
/// Reads as `for (const auto hub : louder::ActorListeningTo(...))` rather than
/// as the loop it wraps.
class ActorListeningTo {
public:
    ActorListeningTo(const LouderContext* ctx, int32_t conversation, int32_t speaker, int32_t start, int32_t end)
        : ctx_(ctx), conversation_(conversation), speaker_(speaker),
          start_(start), end_(end) {}

    class Iterator {
    public:
        Iterator(const ActorListeningTo &owner, int32_t current)
            : owner_(owner), current_(current) {}

        bool operator==(const Iterator &other) const { return current_ == other.current_; }
        int32_t operator*() const { return current_; }
        Iterator &operator++() {
            current_ = next_actor_listening_to(owner_.ctx_, owner_.conversation_, owner_.speaker_, current_);
            return *this;
        }

    private:
        const ActorListeningTo &owner_;
        int32_t current_;
    };

    [[nodiscard]] Iterator begin() const {
        return Iterator(*this, next_actor_listening_to(ctx_, conversation_, speaker_, start_));
    }
    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }

    /// The engine ends this walk with the value its documentation calls
    /// BLANK_POSITION, which the header does not declare.
private:
    const LouderContext* ctx_;
    int32_t conversation_;
    int32_t speaker_;
    int32_t start_;
    int32_t end_;
};

/// Iterate the speakers an actor speaks as.
///
/// Reads as `for (const auto hub : louder::ActorSpeakAs(...))` rather than
/// as the loop it wraps.
class ActorSpeakAs {
public:
    ActorSpeakAs(const LouderContext* ctx, int32_t conversation, int32_t actor, int32_t start, int32_t end)
        : ctx_(ctx), conversation_(conversation), actor_(actor),
          start_(start), end_(end) {}

    class Iterator {
    public:
        Iterator(const ActorSpeakAs &owner, int32_t current)
            : owner_(owner), current_(current) {}

        bool operator==(const Iterator &other) const { return current_ == other.current_; }
        int32_t operator*() const { return current_; }
        Iterator &operator++() {
            current_ = next_actor_speak_as(owner_.ctx_, owner_.conversation_, owner_.actor_, current_);
            return *this;
        }

    private:
        const ActorSpeakAs &owner_;
        int32_t current_;
    };

    [[nodiscard]] Iterator begin() const {
        return Iterator(*this, next_actor_speak_as(ctx_, conversation_, actor_, start_));
    }
    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }

    /// The engine ends this walk with the value its documentation calls
    /// BLANK_POSITION, which the header does not declare.
private:
    const LouderContext* ctx_;
    int32_t conversation_;
    int32_t actor_;
    int32_t start_;
    int32_t end_;
};

/// Iterate the actors speaking as a speaker.
///
/// Reads as `for (const auto hub : louder::ActorSpeakingAs(...))` rather than
/// as the loop it wraps.
class ActorSpeakingAs {
public:
    ActorSpeakingAs(const LouderContext* ctx, int32_t conversation, int32_t speaker, int32_t start, int32_t end)
        : ctx_(ctx), conversation_(conversation), speaker_(speaker),
          start_(start), end_(end) {}

    class Iterator {
    public:
        Iterator(const ActorSpeakingAs &owner, int32_t current)
            : owner_(owner), current_(current) {}

        bool operator==(const Iterator &other) const { return current_ == other.current_; }
        int32_t operator*() const { return current_; }
        Iterator &operator++() {
            current_ = next_actor_speaking_as(owner_.ctx_, owner_.conversation_, owner_.speaker_, current_);
            return *this;
        }

    private:
        const ActorSpeakingAs &owner_;
        int32_t current_;
    };

    [[nodiscard]] Iterator begin() const {
        return Iterator(*this, next_actor_speaking_as(ctx_, conversation_, speaker_, start_));
    }
    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }

    /// The engine ends this walk with the value its documentation calls
    /// BLANK_POSITION, which the header does not declare.
private:
    const LouderContext* ctx_;
    int32_t conversation_;
    int32_t speaker_;
    int32_t start_;
    int32_t end_;
};

/// Iterate the hubs in a channel.
///
/// Reads as `for (const auto hub : louder::HubByChannel(...))` rather than
/// as the loop it wraps.
class HubByChannel {
public:
    HubByChannel(const LouderContext* ctx, int32_t conversation, int32_t channel, int32_t start, int32_t end)
        : ctx_(ctx), conversation_(conversation), channel_(channel),
          start_(start), end_(end) {}

    class Iterator {
    public:
        Iterator(const HubByChannel &owner, int32_t current)
            : owner_(owner), current_(current) {}

        bool operator==(const Iterator &other) const { return current_ == other.current_; }
        int32_t operator*() const { return current_; }
        Iterator &operator++() {
            current_ = next_hub_by_channel(owner_.ctx_, owner_.conversation_, owner_.channel_, current_);
            return *this;
        }

    private:
        const HubByChannel &owner_;
        int32_t current_;
    };

    [[nodiscard]] Iterator begin() const {
        return Iterator(*this, next_hub_by_channel(ctx_, conversation_, channel_, start_));
    }
    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }

    /// The engine ends this walk with the value its documentation calls
    /// BLANK_POSITION, which the header does not declare.
private:
    const LouderContext* ctx_;
    int32_t conversation_;
    int32_t channel_;
    int32_t start_;
    int32_t end_;
};

/// Iterate the hubs with a priority within `[priority_min, priority_max]`.
///
/// Reads as `for (const auto hub : louder::HubByPriority(...))` rather than
/// as the loop it wraps.
class HubByPriority {
public:
    HubByPriority(const LouderContext* ctx, int32_t conversation, int32_t priority_min, int32_t priority_max, int32_t start, int32_t end)
        : ctx_(ctx), conversation_(conversation), priority_min_(priority_min), priority_max_(priority_max),
          start_(start), end_(end) {}

    class Iterator {
    public:
        Iterator(const HubByPriority &owner, int32_t current)
            : owner_(owner), current_(current) {}

        bool operator==(const Iterator &other) const { return current_ == other.current_; }
        int32_t operator*() const { return current_; }
        Iterator &operator++() {
            current_ = next_hub_by_priority(owner_.ctx_, owner_.conversation_, owner_.priority_min_, owner_.priority_max_, current_);
            return *this;
        }

    private:
        const HubByPriority &owner_;
        int32_t current_;
    };

    [[nodiscard]] Iterator begin() const {
        return Iterator(*this, next_hub_by_priority(ctx_, conversation_, priority_min_, priority_max_, start_));
    }
    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }

    /// The engine ends this walk with the value its documentation calls
    /// BLANK_POSITION, which the header does not declare.
private:
    const LouderContext* ctx_;
    int32_t conversation_;
    int32_t priority_min_;
    int32_t priority_max_;
    int32_t start_;
    int32_t end_;
};

/// Iterate the hubs matching certain flags.
///
/// Reads as `for (const auto hub : louder::HubFromFlags(...))` rather than
/// as the loop it wraps.
class HubFromFlags {
public:
    HubFromFlags(const LouderContext* ctx, int32_t conversation, HubFlags include, int32_t exact, int32_t start, int32_t end)
        : ctx_(ctx), conversation_(conversation), include_(include), exact_(exact),
          start_(start), end_(end) {}

    class Iterator {
    public:
        Iterator(const HubFromFlags &owner, int32_t current)
            : owner_(owner), current_(current) {}

        bool operator==(const Iterator &other) const { return current_ == other.current_; }
        int32_t operator*() const { return current_; }
        Iterator &operator++() {
            current_ = next_hub_from_flags(owner_.ctx_, owner_.conversation_, owner_.include_, owner_.exact_, current_);
            return *this;
        }

    private:
        const HubFromFlags &owner_;
        int32_t current_;
    };

    [[nodiscard]] Iterator begin() const {
        return Iterator(*this, next_hub_from_flags(ctx_, conversation_, include_, exact_, start_));
    }
    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }

    /// The engine ends this walk with the value its documentation calls
    /// BLANK_POSITION, which the header does not declare.
private:
    const LouderContext* ctx_;
    int32_t conversation_;
    HubFlags include_;
    int32_t exact_;
    int32_t start_;
    int32_t end_;
};

/// Iterate the hubs containing all, or exactly, the given tags.
///
/// Reads as `for (const auto hub : louder::HubFromTags(...))` rather than
/// as the loop it wraps.
class HubFromTags {
public:
    HubFromTags(const LouderContext* ctx, int32_t conversation, const int32_t* include, int32_t include_count, int32_t exact, int32_t start, int32_t end)
        : ctx_(ctx), conversation_(conversation), include_(include), include_count_(include_count), exact_(exact),
          start_(start), end_(end) {}

    class Iterator {
    public:
        Iterator(const HubFromTags &owner, int32_t current)
            : owner_(owner), current_(current) {}

        bool operator==(const Iterator &other) const { return current_ == other.current_; }
        int32_t operator*() const { return current_; }
        Iterator &operator++() {
            current_ = next_hub_from_tags(owner_.ctx_, owner_.conversation_, owner_.include_, owner_.include_count_, owner_.exact_, current_);
            return *this;
        }

    private:
        const HubFromTags &owner_;
        int32_t current_;
    };

    [[nodiscard]] Iterator begin() const {
        return Iterator(*this, next_hub_from_tags(ctx_, conversation_, include_, include_count_, exact_, start_));
    }
    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }

    /// The engine ends this walk with the value its documentation calls
    /// BLANK_POSITION, which the header does not declare.
private:
    const LouderContext* ctx_;
    int32_t conversation_;
    const int32_t* include_;
    int32_t include_count_;
    int32_t exact_;
    int32_t start_;
    int32_t end_;
};

/// Iterate the hubs heard by a speaker.
///
/// Reads as `for (const auto hub : louder::HubHeardBy(...))` rather than
/// as the loop it wraps.
class HubHeardBy {
public:
    HubHeardBy(const LouderContext* ctx, int32_t conversation, int32_t speaker, int32_t start, int32_t end)
        : ctx_(ctx), conversation_(conversation), speaker_(speaker),
          start_(start), end_(end) {}

    class Iterator {
    public:
        Iterator(const HubHeardBy &owner, int32_t current)
            : owner_(owner), current_(current) {}

        bool operator==(const Iterator &other) const { return current_ == other.current_; }
        int32_t operator*() const { return current_; }
        Iterator &operator++() {
            current_ = next_hub_heard_by(owner_.ctx_, owner_.conversation_, owner_.speaker_, current_);
            return *this;
        }

    private:
        const HubHeardBy &owner_;
        int32_t current_;
    };

    [[nodiscard]] Iterator begin() const {
        return Iterator(*this, next_hub_heard_by(ctx_, conversation_, speaker_, start_));
    }
    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }

    /// The engine ends this walk with the value its documentation calls
    /// BLANK_POSITION, which the header does not declare.
private:
    const LouderContext* ctx_;
    int32_t conversation_;
    int32_t speaker_;
    int32_t start_;
    int32_t end_;
};

/// Iterate the recipients of a hub.
///
/// Reads as `for (const auto hub : louder::HubRecipient(...))` rather than
/// as the loop it wraps.
class HubRecipient {
public:
    HubRecipient(const LouderContext* ctx, int32_t conversation, int32_t position, int32_t start, int32_t end)
        : ctx_(ctx), conversation_(conversation), position_(position),
          start_(start), end_(end) {}

    class Iterator {
    public:
        Iterator(const HubRecipient &owner, int32_t current)
            : owner_(owner), current_(current) {}

        bool operator==(const Iterator &other) const { return current_ == other.current_; }
        int32_t operator*() const { return current_; }
        Iterator &operator++() {
            current_ = next_hub_recipient(owner_.ctx_, owner_.conversation_, owner_.position_, current_);
            return *this;
        }

    private:
        const HubRecipient &owner_;
        int32_t current_;
    };

    [[nodiscard]] Iterator begin() const {
        return Iterator(*this, next_hub_recipient(ctx_, conversation_, position_, start_));
    }
    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }

    /// The engine ends this walk with the value its documentation calls
    /// BLANK_POSITION, which the header does not declare.
private:
    const LouderContext* ctx_;
    int32_t conversation_;
    int32_t position_;
    int32_t start_;
    int32_t end_;
};

/// Iterate the hubs spoken by a speaker.
///
/// Reads as `for (const auto hub : louder::HubSpokenBy(...))` rather than
/// as the loop it wraps.
class HubSpokenBy {
public:
    HubSpokenBy(const LouderContext* ctx, int32_t conversation, int32_t speaker, int32_t start, int32_t end)
        : ctx_(ctx), conversation_(conversation), speaker_(speaker),
          start_(start), end_(end) {}

    class Iterator {
    public:
        Iterator(const HubSpokenBy &owner, int32_t current)
            : owner_(owner), current_(current) {}

        bool operator==(const Iterator &other) const { return current_ == other.current_; }
        int32_t operator*() const { return current_; }
        Iterator &operator++() {
            current_ = next_hub_spoken_by(owner_.ctx_, owner_.conversation_, owner_.speaker_, current_);
            return *this;
        }

    private:
        const HubSpokenBy &owner_;
        int32_t current_;
    };

    [[nodiscard]] Iterator begin() const {
        return Iterator(*this, next_hub_spoken_by(ctx_, conversation_, speaker_, start_));
    }
    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }

    /// The engine ends this walk with the value its documentation calls
    /// BLANK_POSITION, which the header does not declare.
private:
    const LouderContext* ctx_;
    int32_t conversation_;
    int32_t speaker_;
    int32_t start_;
    int32_t end_;
};

/// Iterate the tags of a position.
///
/// Reads as `for (const auto hub : louder::HubTag(...))` rather than
/// as the loop it wraps.
class HubTag {
public:
    HubTag(const LouderContext* ctx, int32_t conversation, int32_t position, int32_t start, int32_t end)
        : ctx_(ctx), conversation_(conversation), position_(position),
          start_(start), end_(end) {}

    class Iterator {
    public:
        Iterator(const HubTag &owner, int32_t current)
            : owner_(owner), current_(current) {}

        bool operator==(const Iterator &other) const { return current_ == other.current_; }
        int32_t operator*() const { return current_; }
        Iterator &operator++() {
            current_ = next_hub_tag(owner_.ctx_, owner_.conversation_, owner_.position_, current_);
            return *this;
        }

    private:
        const HubTag &owner_;
        int32_t current_;
    };

    [[nodiscard]] Iterator begin() const {
        return Iterator(*this, next_hub_tag(ctx_, conversation_, position_, start_));
    }
    [[nodiscard]] Iterator end() const { return Iterator(*this, end_); }

    /// The engine ends this walk with the value its documentation calls
    /// BLANK_POSITION, which the header does not declare.
private:
    const LouderContext* ctx_;
    int32_t conversation_;
    int32_t position_;
    int32_t start_;
    int32_t end_;
};

} // namespace louder

#endif // LOUDER_HPP

