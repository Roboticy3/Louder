// A walk through the engine, run against the real library.
//
//   g++ -std=c++20 -I. walk.cpp louder_rust.dll.lib -o walk.exe
//
// Every line below goes through the same C boundary the header describes, so
// this is the cheapest way to tell a binding that is right from one that merely
// compiles. The values it asserts are the ones the engine's own C++ tests
// assert, which is the point: two languages, one set of answers.

#include <cstdio>
#include <vector>

#include "louder.hpp"

static int checks = 0;
static int failures = 0;

static void check(bool ok, const char *label) {
    checks += 1;
    if (!ok) {
        failures += 1;
    }
    std::printf("  %s %s\n", ok ? "ok  " : "FAIL", label);
}

static const char CONFIG[] =
    "[{\"type\":\"speaker\",\"name\":\"A\",\"flags\":1}]";

static const char CONVERSATION[] =
    "["
    "{\"type\":\"hub\",\"conversation_id\":0,\"speaker\":\"A\",\"text\":\"Hello\","
    "\"tags\":\"start main\",\"flags\":0},"
    "{\"type\":\"hub\",\"conversation_id\":5,\"speaker\":\"A\",\"text\":\"World\",\"flags\":0},"
    "{\"type\":\"hub\",\"conversation_id\":9,\"text\":\"The end\",\"callbacks\":\"exit\",\"flags\":0},"
    "{\"type\":\"connection\",\"from_id\":0,\"from_port\":0,\"to_id\":5},"
    "{\"type\":\"connection\",\"from_id\":5,\"from_port\":0,\"to_id\":9}"
    "]";

int main() {
    louder::Louder engine;

    std::puts("names");
    const louder::Name id = engine.name("alpha");
    check(id != LOUDER_NAME_NONE, "a name is assigned");
    check(engine.text(id) == "alpha", "a name round-trips");
    check(engine.name("alpha") == id, "declaring twice gives the same id");
    check(engine.lookup("never declared") == LOUDER_NAME_NONE,
          "an undeclared name does not resolve");

    std::puts("documents");
    engine.load_config(CONFIG);
    check(true, "the speaker config loads");
    const louder::Name conversation = engine.name("under test");
    engine.load_conversation(conversation, CONVERSATION);
    check(true, "the conversation loads");
    try {
        engine.load_conversation(conversation, CONVERSATION);
        check(false, "loading twice is refused");
    } catch (const louder::LoadFailure &) {
        check(true, "loading twice is refused");
    }
    try {
        engine.load_conversation(engine.name("garbage"), "not json at all");
        check(false, "malformed JSON is refused");
    } catch (const louder::LoadFailure &refused) {
        check(refused.reason() == louder::Failure::MALFORMED_JSON,
              "malformed JSON is refused");
    }

    check(engine.text(get_hub_text(engine.handle(), conversation, 0)) == "Hello",
          "hub 0 reads back");
    check(engine.text(get_hub_text(engine.handle(), conversation, 9)) == "The end",
          "hub 9 reads back");

    std::vector<louder::Position> walked;
    for (const louder::Position hub :
         louder::HubByChannel(engine.handle(), conversation,
                              engine.name("main"), -1, -1)) {
        walked.push_back(hub);
    }
    check((walked == std::vector<louder::Position>{0, 5, 9}),
          "hubs iterate in ascending order");

    std::puts("actors");
    const ActorFlags no_flags{0};
    const louder::Actor actor = add_actor(engine.handle(), conversation, 0,
                                          no_flags, nullptr, 0, nullptr, 0);
    check(actor != LOUDER_ACTOR_NONE, "an actor is added");
    check(get_actor_position(engine.handle(), conversation, actor) == 0,
          "the actor starts at 0");
    check(select_choice(engine.handle(), conversation, actor, 0) == 0,
          "the choice is taken");
    check(get_actor_position(engine.handle(), conversation, actor) == 5,
          "the actor moved to hub 5");

    std::puts("story state");
    engine.set_int(conversation, "mood", 7);
    check(engine.get_int(conversation, "mood") == 7, "an integer round-trips");
    check(engine.get_int(conversation, "unset") == 0,
          "an unwritten variable reads as 0");
    engine.set_text(conversation, "who", "A");
    check(engine.get_text(conversation, "who") == "A",
          "a text variable round-trips through a name");
    check(engine.state_json(conversation).find("\"mood\":7") != std::string::npos,
          "the state JSON carries the value");

    flush_events(engine.handle());

    std::puts("");
    if (failures == 0) {
        std::printf("all %d checks passed\n", checks);
        return 0;
    }
    std::printf("%d FAILURES of %d\n", failures, checks);
    return 1;
}