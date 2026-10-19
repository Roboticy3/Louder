//! A walk through the engine, run against the real library.
//!
//! `cargo run --example walk`
//!
//! Every line below goes through the same C boundary the header describes, so
//! this is the cheapest way to tell a binding that is right from one that
//! merely compiles. The values it asserts are the ones the engine's own C++
//! tests assert, which is the point: two languages, one set of answers.

use louder::ffi::LouderVariantArm;
use louder::Louder;

const CONFIG: &str = r#"[{"type":"speaker","name":"A","flags":1}]"#;

const CONVERSATION: &str = r#"[
  {"type":"hub","conversation_id":0,"speaker":"A","text":"Hello","tags":"start main","flags":0},
  {"type":"hub","conversation_id":5,"speaker":"A","text":"World","flags":0},
  {"type":"hub","conversation_id":9,"text":"The end","callbacks":"exit","flags":0},
  {"type":"connection","from_id":0,"from_port":0,"to_id":5},
  {"type":"connection","from_id":5,"from_port":0,"to_id":9}
]"#;

fn main() {
    let mut checks = 0usize;
    let mut failures: Vec<String> = Vec::new();
    let mut check = |ok: bool, label: &str| {
        checks += 1;
        if !ok {
            failures.push(label.to_string());
        }
        println!("  {} {}", if ok { "ok  " } else { "FAIL" }, label);
    };

    println!("names");
    let mut engine = Louder::new().expect("a context");
    let id = engine.declare("alpha");
    check(id != louder::ffi::LOUDER_NAME_NONE, "a name is assigned");
    check(engine.text(id) == "alpha", "a name round-trips");
    check(engine.declare("alpha") == id, "declaring twice gives the same id");
    check(engine.lookup("never declared").is_none(), "an undeclared name does not resolve");
    check(
        engine.text(louder::ffi::LOUDER_NAME_NONE).is_empty(),
        "the empty name reads as empty",
    );

    println!("documents");
    check(engine.load_config(CONFIG).is_ok(), "the speaker config loads");
    check(
        engine.load_conversation("under test", CONVERSATION).is_ok(),
        "the conversation loads",
    );
    check(
        engine.load_conversation("under test", CONVERSATION).is_err(),
        "loading twice is refused",
    );
    check(
        engine.load_conversation("garbage", "not json at all").is_err(),
        "malformed JSON is refused",
    );

    check(engine.hub_text("under test", 0) == "Hello", "hub 0 reads back");
    check(engine.hub_text("under test", 9) == "The end", "hub 9 reads back");
    check(engine.hub_speaker("under test", 0) == "A", "hub 0 is spoken by A");

    let hubs = engine.hubs("under test", "main");
    check(hubs == vec![0, 5, 9], "hubs iterate in ascending order");

    println!("actors");
    let actor = engine.add_actor("under test", 0, &["A"], &["A"]);
    check(actor >= 0, "an actor is added");
    check(engine.actor_position("under test", actor) == 0, "the actor starts at 0");
    check(engine.hub_choice_count("under test", 0, actor) == 1, "hub 0 has one choice");
    check(
        engine.hub_choice_text("under test", 0, actor, 0) == "World",
        "the choice reads as the next hub",
    );

    let kinds: Vec<&str> = engine.drain_events().iter().map(|e| e.kind).collect();
    check(kinds.contains(&"ActorAdded"), "the actor-added event arrived");

    check(engine.choose("under test", actor, 0).is_ok(), "the choice is taken");
    check(engine.actor_position("under test", actor) == 5, "the actor moved to hub 5");
    check(engine.warp("under test", actor, 0), "the actor is warped back");
    check(engine.actor_position("under test", actor) == 0, "the warp landed");
    check(engine.teleport("under test", actor, 9), "the actor is teleported");
    check(engine.actor_position("under test", actor) == 9, "the teleport landed");

    let speaking = engine.actors_speaking_as("under test", "A");
    check(speaking == vec![actor], "speaking-as iteration finds the actor");
    check(
        engine.speakers_of("under test", actor) == vec!["A".to_string()],
        "the actor speaks as A",
    );
    check(
        engine.tags_of("under test", 0) == vec!["start".to_string(), "main".to_string()],
        "hub 0 carries its tags",
    );

    println!("story state");
    engine.set_int("under test", "mood", 7);
    check(engine.get_int("under test", "mood") == 7, "an integer round-trips");
    check(
        engine.get_int("under test", "unset") == 0,
        "an unwritten variable reads as 0",
    );
    engine.set_name("under test", "who", "A");
    check(
        engine.get_text("under test", "who") == "A",
        "a text variable round-trips through a name",
    );

    let state = engine.state_json("under test").expect("the state as JSON");
    check(state.contains("\"mood\":7"), "the state JSON carries the value");
    let snapshot = engine.snapshot_json("under test").expect("the snapshot as JSON");
    check(snapshot.contains("actors"), "the snapshot carries the actors");

    println!("a variant, read as the kind it is");
    let conversation = engine.declare("under test");
    let key = engine.declare("unset");
    let variant = unsafe { louder::ffi::get_story_state(engine.handle(), conversation, key) };
    check(
        matches!(variant.arm(), Some(LouderVariantArm::int(0))),
        "an unwritten variant reads as a zero integer",
    );

    println!();
    if failures.is_empty() {
        println!("all {checks} checks passed");
    } else {
        println!("{} FAILURES of {checks}:", failures.len());
        for name in &failures {
            println!("  {name}");
        }
        std::process::exit(1);
    }
}
