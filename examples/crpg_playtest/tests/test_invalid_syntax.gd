extends Node

## Checks that malformed expressions are rejected by the parser.

var _failures := 0

func _ready() -> void:
	_check(Evaluator.parse("1 +") != null, "a trailing operator does not parse")
	_check(Evaluator.parse("a # b") != null, "an invalid character does not parse")
	_check(Evaluator.parse("$ in the middle") != null, "a stray dollar does not parse")
	_check(Evaluator.parse("1..2") != null, "a malformed number does not parse")
	_check(Evaluator.parse("") != null, "an empty expression does not parse")
	_check(Evaluator.parse("foo(") != null, "an unclosed call does not parse")
	_check(Evaluator.tokenize("a # b").is_empty(), "tokenizing invalid input fails")

	if _failures > 0:
		get_tree().quit(1)


func _check(condition: bool, message: String) -> void:
	print("%s %s" % ["PASS" if condition else "FAIL", message])
	if !condition:
		_failures += 1
