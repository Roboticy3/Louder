/* A walk through the engine, run against the real library.
 *
 *   gcc -I. walk.c -L. -llouder_rust -o walk
 *
 * Every line below goes through the same C boundary the header describes, so
 * this is the cheapest way to tell a binding that is right from one that merely
 * compiles. The values it asserts are the ones the engine's own C++ tests
 * assert, which is the point: two languages, one set of answers.
 */

#include <stdio.h>
#include <string.h>

#include "louder.h"

static int checks = 0;
static int failures = 0;
static const LouderContext *reader = NULL;

static void check(int ok, const char *label) {
    checks += 1;
    if (!ok) {
        failures += 1;
    }
    printf("  %s %s\n", ok ? "ok  " : "FAIL", label);
}

/* Whether a name reads back as exactly this text. */
static int equals(LouderName name, const char *expected) {
    LouderBuffer text = {0};
    int same;
    if (!louder_read_name(reader, name, &text)) {
        return 0;
    }
    same = text.length == strlen(expected) && memcmp(text.data, expected, text.length) == 0;
    louder_buffer_free(&text);
    return same;
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

int main(void) {
    LouderContext *engine;
    LouderName conversation, id, mood;
    LouderActor actor;
    LouderVariant value;
    LouderBuffer state = {0};
    LouderHubByChannel hubs;
    LouderPosition hub;

    printf("names\n");
    engine = init_louder();
    reader = engine;
    check(engine != NULL, "a context opens");

    id = louder_name(engine, "alpha");
    check(id != LOUDER_NAME_NONE, "a name is assigned");
    check(equals(id, "alpha"), "a name round-trips");
    check(louder_name(engine, "alpha") == id, "declaring twice gives the same id");

    printf("documents\n");
    check(louder_load_conversation_config_text(engine, CONFIG) == LOUDER_LOAD_OK,
          "the speaker config loads");
    check(louder_load_conversation_json_text(engine, louder_name(engine, "under test"),
                                             CONVERSATION) == LOUDER_LOAD_OK,
          "the conversation loads");
    check(louder_load_conversation_json_text(engine, louder_name(engine, "garbage"),
                                             "not json at all") == LOUDER_LOAD_MALFORMED_JSON,
          "malformed JSON is refused");

    conversation = louder_name(engine, "under test");
    check(equals(get_hub_text(engine, conversation, 0), "Hello"), "hub 0 reads back");
    check(equals(get_hub_text(engine, conversation, 9), "The end"), "hub 9 reads back");

    hubs = louder_hub_by_channel(engine, conversation, louder_name(engine, "main"), -1, -1);
    check(louder_hub_by_channel_next(&hubs, &hub) && hub == 0, "the walk starts at hub 0");
    check(louder_hub_by_channel_next(&hubs, &hub) && hub == 5, "then hub 5");
    check(louder_hub_by_channel_next(&hubs, &hub) && hub == 9, "then hub 9");
    check(!louder_hub_by_channel_next(&hubs, &hub), "and then it stops");

    printf("actors\n");
    actor = add_actor(engine, conversation, 0, (ActorFlags){0}, NULL, 0, NULL, 0);
    check(actor != LOUDER_ACTOR_NONE, "an actor is added");
    check(get_actor_position(engine, conversation, actor) == 0, "the actor starts at 0");
    check(select_choice(engine, conversation, actor, 0) == 0, "the choice is taken");
    check(get_actor_position(engine, conversation, actor) == 5, "the actor moved to hub 5");

    printf("story state\n");
    mood = louder_name(engine, "mood");
    set_story_state(engine, conversation, mood, make_variant_int(7));
    value = get_story_state(engine, conversation, mood);
    check(value.tag == Int && value.body.int_ == 7, "an integer round-trips");
    check(louder_read_state_json(engine, conversation, &state), "the state reads back");
    check(strstr((const char *)state.data, "\"mood\":7") != NULL,
          "the state JSON carries the value");
    louder_buffer_free(&state);

    flush_events(engine);
    deinit_louder(engine);

    printf("\n");
    if (failures == 0) {
        printf("all %d checks passed\n", checks);
        return 0;
    }
    printf("%d FAILURES of %d\n", failures, checks);
    return 1;
}