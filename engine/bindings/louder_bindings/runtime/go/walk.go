// A walk through the engine, run against the real library.
//
// Every line below goes through the same C boundary the header describes, so
// this is the cheapest way to tell a binding that is right from one that merely
// compiles. The values it asserts are the ones the engine's own C++ tests
// assert, which is the point: two languages, one set of answers.

package main

import (
	"fmt"
	"os"
	"strings"

	"louder/louder"
)

var checks = 0
var failures = 0

func check(ok bool, label string) {
	checks++
	if !ok {
		failures++
	}
	mark := "ok  "
	if !ok {
		mark = "FAIL"
	}
	fmt.Printf("  %s %s\n", mark, label)
}

const config = `[{"type":"speaker","name":"A","flags":1}]`

const conversation = `[
  {"type":"hub","conversation_id":0,"speaker":"A","text":"Hello","tags":"start main","flags":0},
  {"type":"hub","conversation_id":5,"speaker":"A","text":"World","flags":0},
  {"type":"hub","conversation_id":9,"text":"The end","callbacks":"exit","flags":0},
  {"type":"connection","from_id":0,"from_port":0,"to_id":5},
  {"type":"connection","from_id":5,"from_port":0,"to_id":9}
]`

func main() {
	engine, err := louder.Open()
	check(err == nil, "a context opens")
	if err != nil {
		os.Exit(1)
	}
	defer engine.Close()

	fmt.Println("names")
	id := engine.Name("alpha")
	check(id != louder.LOUDER_NAME_NONE, "a name is assigned")
	check(engine.Text(id) == "alpha", "a name round-trips")
	check(engine.Name("alpha") == id, "declaring twice gives the same id")
	check(engine.Lookup("never declared") == louder.LOUDER_NAME_NONE,
		"an undeclared name does not resolve")

	fmt.Println("documents")
	check(engine.LoadConfig(config) == nil, "the speaker config loads")
	check(engine.LoadConversation(engine.Name("under test"), conversation) == nil,
		"the conversation loads")
	check(engine.LoadConversation(engine.Name("under test"), conversation) != nil,
		"loading twice is refused")
	check(engine.LoadConversation(engine.Name("garbage"), "not json at all") != nil,
		"malformed JSON is refused")

	conv := engine.Name("under test")
	check(engine.Text(louder.GetHubText(engine.Handle(), conv, 0)) == "Hello", "hub 0 reads back")
	check(engine.Text(louder.GetHubText(engine.Handle(), conv, 9)) == "The end", "hub 9 reads back")

	var hubs []int32
	for hub := range engine.HubsByChannel(conv, engine.Name("main")) {
		hubs = append(hubs, int32(hub))
	}
	check(len(hubs) == 3 && hubs[0] == 0 && hubs[1] == 5 && hubs[2] == 9,
		"hubs iterate in ascending order")

	fmt.Println("actors")
	actor := louder.AddActor(engine.Handle(), conv, 0, louder.ActorFlags{}, nil, 0, nil, 0)
	check(actor != louder.LOUDER_ACTOR_NONE, "an actor is added")
	check(louder.GetActorPosition(engine.Handle(), conv, actor) == 0, "the actor starts at 0")
	check(louder.SelectChoice(engine.Handle(), conv, actor, 0) == 0, "the choice is taken")
	check(louder.GetActorPosition(engine.Handle(), conv, actor) == 5, "the actor moved to hub 5")

	fmt.Println("story state")
	engine.SetInt(conv, "mood", 7)
	check(engine.Variant(conv, "mood").Int == 7, "an integer round-trips")
	engine.SetText(conv, "who", "A")
	state, err := engine.StateJSON(conv)
	check(err == nil, "the state reads back")
	check(strings.Contains(state, `"mood":7`), "the state JSON carries the value")

	fmt.Println()
	if failures == 0 {
		fmt.Printf("all %d checks passed\n", checks)
		return
	}
	fmt.Printf("%d FAILURES of %d\n", failures, checks)
	os.Exit(1)
}
