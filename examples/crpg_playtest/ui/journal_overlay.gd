class_name JournalOverlay extends Control

@export var scroll_container: ScrollContainer
@export var container: VBoxContainer
@export var journal_container: VBoxContainer
@export var journal_log: ConversationLog
@export var close_button: Button

var scroll: ScrollContainer:
	get:
		return scroll_container
var vbox: VBoxContainer:
	get:
		return container if container != null else journal_container
var log: ConversationLog:
	get:
		return journal_log

signal close_requested

func _ready() -> void:
	if scroll_container == null:
		scroll_container = get_node_or_null("ScrollContainer") as ScrollContainer
	if container == null and scroll_container != null:
		container = scroll_container.get_node_or_null("VBoxContainer") as VBoxContainer
	if journal_container == null:
		journal_container = container
	if container == null and journal_container != null:
		container = journal_container
	if journal_log == null:
		if container != null:
			journal_log = container.get_node_or_null("ConversationLog") as ConversationLog
		if journal_log == null:
			journal_log = get_node_or_null("ConversationLog") as ConversationLog
		if journal_log == null and scroll_container != null:
			journal_log = scroll_container.get_node_or_null("VBoxContainer/ConversationLog") as ConversationLog
	if close_button == null:
		close_button = get_node_or_null("CloseButton") as Button
	if close_button != null and not close_button.pressed.is_connected(_on_close_pressed):
		close_button.pressed.connect(_on_close_pressed)
	visible = false

func _on_close_pressed() -> void:
	visible = false
	close_requested.emit()

func open() -> void:
	visible = true

func close() -> void:
	visible = false

func toggle() -> void:
	visible = not visible

func is_open() -> bool:
	return visible

func is_journal_open() -> bool:
	return visible

var journal_open: bool:
	get:
		return visible
