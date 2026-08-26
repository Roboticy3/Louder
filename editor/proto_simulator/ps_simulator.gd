class_name PSSimulator extends Node

const HUB_CVID := "conversation_id"
const HUB_TXTC := "text"
const HUB_SPKR := "speaker"
const HUB_COND := "conditions"
const HUB_STMT := "statements"
const HUB_DSKL := "dice_skill"
const HUB_DLVL := "dice_level"
const HUB_SCHK := "start"
const HUB_ACHK := "auto"
const HUB_CHCS := "choices"
const HUB_BRCH := "branch"

const CON_TOID := "to_id"
const CON_FCHC := "from_choice"

@export var conversation_path:="user://test.json"
@export var conversation: PSConversation

@export var reset_state_on_ready := false
@export var state:PSDialogueState

var e := Expression.new()
var recessive := {}
var dominant := {}
var last_branch_traversal := ""

@export var controller_group := &"ps_controller"

### Getters
## These functions do not change the state of the simulator.

func get_position(branch:="") -> int:
	var b:PSDialogueBranch = state.branches.get(branch)
	if !b:
		push_error("PSSimulator::get_position invalid branch ", branch, " (options: ", state.branches.keys(), ")")
		return -1
	return b.position

func list_choices_data(filter=null, branch:="", evaluate_conditions:=true) -> Array:
	var choices := conversation.get_connections_from(get_position(branch))
	if evaluate_conditions: choices = _evaluate_choices(choices, branch)
	if filter is Callable:
		choices = choices.filter(filter)
	return choices

func is_auto(from_position:=get_position()) -> bool:
	var h = conversation.hubs_by_id.get(from_position)
	if h == null:
		push_error("Hub ", from_position, " does not exist!")
		return false
	
	return h.get(HUB_ACHK, false)

func is_start(from_position:=get_position()) -> bool:
	var h = conversation.hubs_by_id.get(from_position)
	if h == null:
		push_error("Hub ", from_position, " does not exist!")
		return false
	
	return h.get(HUB_SCHK, false)

func get_property(from_position:int, property:String, default=null):
	var h = conversation.hubs_by_id.get(from_position, {})
	return str(h.get(property, default))

func get_current_property(property:String, branch:="", default=null):
	return get_property(get_position(branch), property, default)

func merge_current_into(data:Dictionary, branch:=""):
	var h:Dictionary = conversation.hubs_by_id.get(get_position(branch), {})
	for k in h:
		if h[k]:
			data[k] = h[k]

func roll_dice() -> int:
	return (randi() % 6) + (randi() % 6) + 2

### Actions
## These functions are internal actions of the simulator and should not be
## called externally.

func _ready():
	last_branch_traversal = ""
	recessive = {}
	dominant = {}
	if !state or reset_state_on_ready:
		_reset_state()
	var c := PSConversation.new()
	c.load_from_json(conversation_path)
	
	if c.hubs_by_id.is_empty():
		return
	
	conversation = c
	
	var start := int(c.hubs_by_id.keys()[0])
	for id in c.hubs_by_id:
		if is_start(id):
			start = id
			break
		
	warp(start)
	
	update_controllers()

func _reset_state():
	state = PSDialogueState.new()

func _on_conversation_changed(new_path:String):
	conversation_path = new_path
	_ready()

func _evaluate_choices(choices: Array, branch:="") -> Array:
	var valid_choices := []
	for c in choices:
		if _evaluate_choice_single(c, branch):
			valid_choices.append(c)

	return valid_choices

func _count_conditions(conditions:String) -> int:
	if conditions.is_empty():
		return 0
	return conditions.count("\n") + 1

func _evaluate_choice_single(c:Dictionary, branch:="") -> bool:
	var b:PSDialogueBranch = state.branches.get(branch)
	if !b:
		push_error("PSSimulator::_evaluate_choice_single invalid branch ", branch, " (options: ", state.branches.keys(), ")")
		return false
	
	var to_id = int(c.get(CON_TOID))
	var hub = conversation.hubs_by_id[to_id]
	if !hub.get(HUB_COND):
		return true
	
	var conditions := str(hub.get(HUB_COND))
	c[HUB_COND] = conditions
	
	var conditions_arr := conditions.split("\n", false)
	for condition in conditions_arr:
		var error := e.parse(condition)
		if error != OK:
			push_error("Condition ", condition, " \"", e.get_error_text(), "\". Treating as false")
			return false
		var result = e.execute([], state.scope)
		print("Condition ", condition, " evaluated to ", result, ". Resulting scope: ", state.scope)
		if !result:
			return false
	return true

func _evaluate_hub(h:Dictionary, branch:="") -> void:
	_evaluate_hub_expressions(h, branch)
	_evaluate_dice_checks(h, branch)
	
	if !recessive.is_empty() && !is_auto(get_position(branch)):
		h.merge(recessive)
	if !dominant.is_empty():
		h.merge(dominant, true)

func _evaluate_hub_expressions(h:Dictionary, branch:="") -> void:
	var b:PSDialogueBranch = state.branches.get(branch)
	if !b:
		push_error("PSSimulator::_evaluate_hub_expressions invalid branch ", branch, " (options: ", state.branches.keys(), ")")
		return
	
	if !h.get(HUB_STMT):
		return
	var expressions = str(h.get(HUB_STMT))
	var expressions_arr = expressions.split("\n")
	
	#three group regex to manually parse assignments:
	#(gap) (identifier) (gap) (equals) (expression)
	var re := RegEx.create_from_string(r"\s*([a-zA-Z][a-zA-Z0-9_]*)\s*(=)\s*(.*)")
	for expression in expressions_arr:
		var matches := re.search(expression)
		if !matches:
			push_error("Assignment ", expression, " confused the parser.")
			continue
		
		if matches.strings.size() < 4:
			push_error("Assignment ", expression, " is incomplete.")
			continue
		
		var error := e.parse(matches.strings[3])
		if error != OK:
			push_warning("Expression ", matches.strings[3], " \"", e.get_error_text(), "\". Skipping..")
		state.scope[matches.strings[1]] = e.execute([], state.scope)
		print("Expression ", matches.strings[3], " evaluated \"", e.get_error_text(), "\". Resulting scope: ", state.scope)
	return

func _evaluate_dice_checks(h:Dictionary, branch:="") -> void:
	var b:PSDialogueBranch = state.branches.get(branch)
	if !b:
		push_error("PSSimulator::_evaluate_dice_checks invalid branch ", branch, " (options: ", state.branches.keys(), ")")
		return
	
	if h.get(HUB_CVID) == null:
		push_error("Hub ", h, " has no conversation_id!")
		return
	
	#get *all* connections from this hub, even if one port repeats.
	var connections = conversation.get_connections_from(int(h.get(HUB_CVID)))
	for c in connections:
		if !(c is Dictionary):
			continue
		
		if c.get(CON_TOID) == null:
			push_error("Connection ", c, " from hub ", h.get(HUB_CVID), " has no to_id!")
			continue

		var check_h = conversation.hubs_by_id.get(int(c.get(CON_TOID)))
		if !(check_h is Dictionary):
			push_error("Connection ", c, " from hub ", h.get(HUB_CVID), " has invalid to_id!")
			continue
		
		if check_h.get(HUB_DSKL) == null or c.get(HUB_DSKL) == "":
			continue

		print("Evaluating dice check for connection ", c, " from hub ", h.get(HUB_CVID))

		var dice_skill := str(check_h.get(HUB_DSKL, ""))
		var skill_level := int(state.scope.get(dice_skill))
		var dice_level := int(check_h.get(HUB_DLVL, 0))
		
		var check_variable := "_%s_%s_%s_dc" % [h.get(HUB_CVID), c.get(CON_TOID), dice_skill]
		_roll_into_outcome(check_variable, skill_level, state.scope)

		#to enact the dice roll, add a condition. This means future calls to
		#list_choices will filter out failed rolls.
		var check_conditions := str(check_h.get(HUB_COND, ""))
		check_conditions += "\n%s >= %s" % [check_variable, dice_level]
		check_h[HUB_COND] = check_conditions
		check_h.erase(HUB_DSKL)

func _roll_into_outcome(outcome_variable:String, skill_level:int, scope:PSVariableScope) -> void:
	var roll := roll_dice()
	var total := roll + skill_level
	scope[outcome_variable] = total
	print("Rolled ", roll, " + ", skill_level, " = ", total, " for ", outcome_variable)

### TRAVERSAL
## Move through dialogue by making a choice on the current node.
## If the resulting choice is invalid, traversal functions return false and
## emit an error.
## If the resulting choice is valid, traversal functions change `position` to
## the node linked by the choice made. If the resulting node has AutoCheck
## enabled, traversal will continue recursively until a non-auto node is
## reached.
func traverse(choice_text: String, filter=null, branch:="", depth:=0) -> bool:
	if depth == 0:
		last_branch_traversal = ""
		recessive = {}
		dominant = {}
	var position := get_position(branch)
	var choice = null
	for c in conversation.get_connections_from(position):
		if str(c.get(CON_FCHC, "")) == choice_text:
			choice = c
			break
	
	if choice == null:
		push_error("Traversed to nonexistent choice: ", choice_text)
		return false
	
	recessive[HUB_TXTC] = choice.get(CON_FCHC)
	var w := warp(int(choice.get(CON_TOID, "")), branch, depth)
	
	update_controllers()
	return w

func traverse_index(i: int, filter=null, branch:="", depth:=0) -> bool:
	if depth == 0:
		last_branch_traversal = ""
		recessive = {}
		dominant = {}
	var choices := list_choices_data(filter, branch)
	if i < 0 or i >= choices.size():
		push_error("Traversed to choice index out of bounds: ", i)
		return false
	
	recessive[HUB_TXTC] = choices[i].get(CON_FCHC)
	var w := warp(int(choices[i].get(CON_TOID, "")), branch, depth)
	
	update_controllers()
	return w

func warp(new_position:int, branch:="", depth:=0) -> bool:
	var b:PSDialogueBranch = state.branches.get(branch)
	if !b:
		push_error("PSSimulator::warp invalid branch ", branch, " (options: ", state.branches.keys(), ")")
		return false
	
	if new_position in conversation.hubs_by_id:
		var h:Dictionary = conversation.hubs_by_id[new_position]
		branch = _fill_branch_traversal(h, new_position, branch)
		_evaluate_hub(h, branch)
		state._on_traverse(new_position)
		
		if is_auto(new_position):
			_fill_dominant_on_auto(branch)
			#consume choices until either all branches are consumed or no choices are left
			var working_branch := branch
			var i = 0
			var found_any := false
			while traverse_index(i,null,branch,depth + 1):
				found_any = true
				if last_branch_traversal.is_empty() || last_branch_traversal == working_branch:
					#branches have been consumed
					#repeatedly travesing on the same branch will cause unexpected behavior
					break 
				working_branch = last_branch_traversal
				i += 1
			
			return found_any
		
		return true
	push_error("Warped to nonexistent location: ", new_position)
	return false

func _fill_dominant_on_auto(branch:=""):
	var position := get_position(branch)
	var w_txtc = conversation.hubs_by_id[position].get(HUB_TXTC)
	if w_txtc:
		dominant[HUB_TXTC] = w_txtc
		var w_spkr = conversation.hubs_by_id[position].get(HUB_SPKR)
		if w_spkr:
			dominant[HUB_SPKR] = w_spkr

func _fill_branch_traversal(h:Dictionary, position:int, branch:="") ->String:
	var working_branch := branch
	
	var h_branch:String = h.get(HUB_BRCH, "")
	if !h_branch.is_empty():
		working_branch = h_branch
	
	var different_branch := false
	if !working_branch.is_empty() and working_branch != branch:
		different_branch = true
		last_branch_traversal = working_branch
	
	var b:PSDialogueBranch = state.branches.get(working_branch)
	var new_branch := false
	if !b:
		new_branch = true
		b = PSDialogueBranch.new()
		state.branches[working_branch] = b
	
	if new_branch or !different_branch: 
		b.position = position
	
	return working_branch

func update_controllers():
	get_tree().call_group(controller_group, &"update")
