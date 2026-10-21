extends Node

## Checks that hub conditions gate movement and hub statements run once.
##
## The conversation built here is the shape that exposed the bug in marrow.json:
## an Auto start hub whose first choice leads to an Auto hub carrying the
## condition "0". The condition is false, so the auto logic must skip that
## choice and carry on to the next one instead of walking into the blocked hub
## and dead-ending on it.

var _failures := 0

var _ctx: LouderContext
var _conversation: int = -1
var _actor: int = -1
var _evaluator: Evaluator

const GATE_CONDITION := "gate == 1"
const VISITS_STATEMENT := "visits += 1"


func _ready() -> void:
	_ctx = LouderContext.new()
	_conversation = _ctx.make_name(&"conditions_test")

	var loaded := _ctx.load_conversation(_conversation, _conversation_document())
	_check(loaded == 0, "the test conversation loaded (code %d)" % loaded)

	_actor = _ctx.add_actor(_conversation, 3, 0, [], [])
	_check(_actor != -1, "an actor was added")

	_evaluator = Evaluator.create(EvaluationScope.create(_conversation, _actor, -1))

	_check(Evaluator.classify(GATE_CONDITION) == 0, "a comparison classifies as a condition")
	_check(Evaluator.classify(VISITS_STATEMENT) == 1, "an assignment classifies as a statement")
	_check(not Evaluator.condition_holds(_ctx, _conversation, "0"), "the condition \"0\" does not hold")
	_check(Evaluator.condition_holds(_ctx, _conversation, "1"), "the condition \"1\" holds")

	_test_auto_target_conditions()
	_test_statement_runs_once()
	_test_conditions_gate_regular_hubs()

	if _failures > 0:
		get_tree().quit(1)


## The start of the marrow bug: hub 0 is Auto and its first choice leads to hub
## 5, an Auto hub whose condition is "0". Auto hubs never *appear*, but their
## conditions still decide whether auto logic may pass through them.
func _test_auto_target_conditions() -> void:
	_evaluator.evaluate_int(_ctx, "gate = 1")

	var indices: Array = Simulator.available_choices(_ctx, _conversation, _actor, 0)
	_check(indices == [1],
		"auto hub 0 offers only choice 1: choice 0 leads to the hub whose condition \"0\" blocks it (got %s)" % str(indices))

	_check(Simulator.force_position(_ctx, _conversation, _actor, 0, -1, 0),
		"forcing the actor through the auto start hub succeeds instead of dead-ending")
	_check(_ctx.get_actor_position(_conversation, _actor) == 3,
		"the auto hub skipped the blocked branch and landed on hub 3 (landed on %d)" % _ctx.get_actor_position(_conversation, _actor))


## A statement runs the first time a hub is covered and never again, and each
## arrival reports the hub's conditions as events.
func _test_statement_runs_once() -> void:
	_check(_evaluator.evaluate_int(_ctx, "visits") == 1,
		"the statement on hub 3 ran once on the first visit")

	_check(_count_condition_events(GATE_CONDITION) == 1,
		"arriving at hub 3 reported its condition as an event")

	_check(Simulator.select_choice(_ctx, _conversation, _actor, 0), "the actor chose from hub 3 to hub 4")
	_check(_ctx.get_actor_position(_conversation, _actor) == 4, "the actor landed on hub 4")

	_ctx.flush_events()
	_check(Simulator.select_choice(_ctx, _conversation, _actor, 0), "the actor chose from hub 4 back to hub 3")
	_check(_ctx.get_actor_position(_conversation, _actor) == 3, "the actor landed back on hub 3")
	_check(_evaluator.evaluate_int(_ctx, "visits") == 1,
		"the statement on hub 3 did not run again on a second visit")
	_check(_count_condition_events(GATE_CONDITION) == 1,
		"the second arrival reported the condition again, still evaluating true")


## With the gate variable back at 0, the condition on the regular hub 3 fails:
## it may not appear, the choice into it is not offered, and choosing it by
## index is refused.
func _test_conditions_gate_regular_hubs() -> void:
	_check(Simulator.select_choice(_ctx, _conversation, _actor, 0),
		"with the gate open, the actor moved on from hub 3 to hub 4")
	_evaluator.evaluate_int(_ctx, "gate = 0")

	_check(not Simulator.hub_appears(_ctx, _conversation, _actor, 3),
		"hub 3 does not appear while its condition fails")
	_check(Simulator.available_choices(_ctx, _conversation, _actor, 4) == [],
		"no choice into the blocked hub 3 is offered from hub 4")
	_check(not Simulator.select_choice(_ctx, _conversation, _actor, 0),
		"selecting the blocked choice by index is refused")
	_check(_ctx.get_actor_position(_conversation, _actor) == 4,
		"the refused choice left the actor where it was")


## A conversation shaped like the one in marrow.json: an Auto start hub whose
## first choice is a dead-ended Auto hub behind the condition "0".
func _conversation_document() -> String:
	return JSON.stringify([
		{"type": "hub", "conversation_id": "0", "tags": "start", "flags": "1", "text": "gate"},
		{"type": "hub", "conversation_id": "5", "flags": "1", "expressions": "0", "text": "(idk..)"},
		{"type": "hub", "conversation_id": "3", "choices": "1", "expressions": GATE_CONDITION + "\n" + VISITS_STATEMENT, "text": "three"},
		{"type": "hub", "conversation_id": "4", "choices": "1", "text": "four"},
		{"type": "connection", "from_id": "0", "from_port": 0, "to_id": "5"},
		{"type": "connection", "from_id": "0", "from_port": 1, "to_id": "3"},
		{"type": "connection", "from_id": "3", "from_port": 0, "to_id": "4"},
		{"type": "connection", "from_id": "4", "from_port": 0, "to_id": "3"},
	])


## How many condition events naming `source` are sitting in the queue.
func _count_condition_events(source: String) -> int:
	var wanted := _ctx.make_name(source)
	var found := 0
	for index in _ctx.get_event_count():
		var event := _ctx.get_event(index)
		if event == null:
			continue
		var outcome := event.get_condition()
		if outcome != null and outcome.get_condition() == wanted:
			found += 1
	return found


func _check(condition: bool, message: String) -> void:
	print("%s %s" % ["PASS" if condition else "FAIL", message])
	if !condition:
		_failures += 1
