class_name CustomGraphCompletionMenu extends ItemList

## Drop-down of completion candidates for a CodeEdit. CodeEdit only draws its
## own completion popup inside the Godot editor itself, so both the menu and
## the insertion of the picked candidate are done here.

var code_edit:CodeEdit
var selected := -1

func _ready() -> void:
	#this property isn't real :(
	#fit_to_longest_item = true
	auto_width = true
	auto_height = true
	item_selected.connect(_on_item_selected)
	item_activated.connect(_on_item_activated)
	hide()

#candidates are the exact texts that will replace the word before the caret.
func open(target:CodeEdit, candidates:Array[String]) -> void:
	code_edit = target
	clear()
	for candidate in candidates:
		add_item(candidate)
	select(0)
	selected = 0
	place()
	show()
	move_to_front()

func close() -> void:
	hide()
	clear()
	code_edit = null
	selected = -1

func _input(event: InputEvent) -> void:
	if !visible:
		return
	if event.is_action_pressed("ui_down"):
		move_selection(1)
	elif event.is_action_pressed("ui_up"):
		move_selection(-1)
	elif event.is_action_pressed("ui_text_completion_accept") or event.is_action_pressed("ui_text_completion_replace"):
		confirm()
	elif event.is_action_pressed("ui_cancel"):
		close()
	else:
		return
	get_viewport().set_input_as_handled()

func move_selection(offset:int) -> void:
	if item_count == 0:
		return
	if selected < 0:
		selected = 0
	else:
		selected = posmod(selected + offset, item_count)
	select(selected)
	ensure_current_is_visible()

func confirm() -> void:
	var target := code_edit
	var insert_text := "" if selected < 0 or selected >= item_count else get_item_text(selected)
	close()
	if is_instance_valid(target) and !insert_text.is_empty():
		replace_word(target, insert_text)
		print("completed with \"", insert_text, "\"")

static func replace_word(target:CodeEdit, insert_text:String) -> void:
	var line := target.get_caret_line()
	var column := target.get_caret_column()
	var line_text := target.get_line(line)
	var start := column - get_word(target).length()
	target.set_line(line, line_text.substr(0, start) + insert_text + line_text.substr(column))
	target.set_caret_line(line)
	target.set_caret_column(start + insert_text.length())
	target.grab_focus()

#The letters, digits and underscores directly left of the caret.
static func get_word(target:CodeEdit) -> String:
	var column := target.get_caret_column()
	var line_text := target.get_line(target.get_caret_line())
	var start := column
	while start > 0 and is_word_char(line_text[start - 1]):
		start -= 1
	return line_text.substr(start, column - start)

static func is_word_char(c:String) -> bool:
	return c == "_" or (c >= "a" and c <= "z") or (c >= "A" and c <= "Z") or (c >= "0" and c <= "9")

#Hangs the menu under the caret, or above it when there is no room below.
func place() -> void:
	var caret := code_edit.get_rect_at_line_column(code_edit.get_caret_line(), code_edit.get_caret_column())
	var top := code_edit.get_screen_position() + Vector2(caret.position)
	var below := top + Vector2(0, caret.size.y)
	var height := get_combined_minimum_size().y
	if below.y + height <= get_viewport_rect().size.y:
		position = below
	else:
		position = top - Vector2(0, height)

func _on_item_selected(index:int) -> void:
	select(index)
	selected = index
	confirm()

func _on_item_activated(index:int) -> void:
	select(index)
	selected = index
	confirm()
