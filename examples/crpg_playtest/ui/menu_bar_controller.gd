class_name MenuBarController extends Control

## General controller for a menu bar and its submenus (journal, etc.).

@export var menu_bar: Control
@export var journal_button: Button
@export var journal_overlay: Control
@export var journal_scroll: ScrollContainer
@export var journal_container: VBoxContainer
@export var journal_log: ConversationLog

## Aliases kept for callers/tests using other names.
var submenu_bar: Control:
	get:
		return menu_bar
var journal_panel: Control:
	get:
		return journal_overlay
var journal_view: Control:
	get:
		return journal_overlay
var menu: Control:
	get:
		return menu_bar

## Optional link to the dialogue whose actor drives the journal.
@export var scrolling_dialogue: ScrollingDialogue

var _submenus: Dictionary = {}
var _active_submenu := ""
var _busy := false
var _toggled_just := false

func _ready() -> void:
	if menu_bar == null:
		menu_bar = get_node_or_null("MenuBar") as Control
		if menu_bar == null:
			menu_bar = get_node_or_null("MenuBarContainer/MenuBar") as Control
	if submenu_bar == null:
		submenu_bar = menu_bar
	if journal_button == null:
		if menu_bar != null:
			journal_button = menu_bar.get_node_or_null("JournalButton") as Button
		if journal_button == null:
			journal_button = get_node_or_null("JournalButton") as Button
		if journal_button == null:
			journal_button = get_node_or_null("MenuBarContainer/MenuBar/JournalButton") as Button
	if journal_overlay == null:
		journal_overlay = get_node_or_null("JournalOverlay") as Control
		if journal_overlay == null:
			journal_overlay = get_node_or_null("Journal") as Control
	if journal_panel == null:
		journal_panel = journal_overlay
	if journal_view == null:
		journal_view = journal_overlay

	# Resolve journal internals from the dedicated overlay when present.
	if journal_overlay is JournalOverlay:
		if journal_scroll == null:
			journal_scroll = (journal_overlay as JournalOverlay).scroll_container
		if journal_container == null:
			journal_container = (journal_overlay as JournalOverlay).container
		if journal_log == null:
			journal_log = (journal_overlay as JournalOverlay).journal_log
	else:
		if journal_scroll == null and journal_overlay != null:
			journal_scroll = journal_overlay.get_node_or_null("ScrollContainer") as ScrollContainer
		if journal_container == null and journal_scroll != null:
			journal_container = journal_scroll.get_node_or_null("VBoxContainer") as VBoxContainer
		if journal_log == null:
			if journal_container != null:
				journal_log = journal_container.get_node_or_null("ConversationLog") as ConversationLog
			if journal_log == null and journal_overlay != null:
				journal_log = journal_overlay.get_node_or_null("ConversationLog") as ConversationLog
				if journal_log == null:
					journal_log = journal_overlay.get_node_or_null("ScrollContainer/VBoxContainer/ConversationLog") as ConversationLog

	if journal_overlay != null:
		journal_overlay.visible = false
		# Keep decorative backgrounds from swallowing clicks.
		for child in journal_overlay.get_children():
			if child is ColorRect:
				(child as Control).mouse_filter = Control.MOUSE_FILTER_IGNORE
		var close_button := _find_close_button(journal_overlay)
		if close_button != null:
			close_button.z_index = 10
			close_button.mouse_filter = Control.MOUSE_FILTER_STOP
			if not close_button.pressed.is_connected(_on_submenu_close_requested):
				close_button.pressed.connect(_on_submenu_close_requested)
		register_submenu("journal", journal_overlay)
		if journal_overlay.has_signal("close_requested"):
			if not journal_overlay.close_requested.is_connected(_on_overlay_close_requested):
				journal_overlay.close_requested.connect(_on_overlay_close_requested)

	if journal_button != null:
		if journal_button.toggle_mode:
			pass
		else:
			journal_button.toggle_mode = true
		if not journal_button.pressed.is_connected(_on_journal_pressed):
			journal_button.pressed.connect(_on_journal_pressed)
		if not journal_button.toggled.is_connected(_on_journal_toggled):
			journal_button.toggled.connect(_on_journal_toggled)

	if scrolling_dialogue == null:
		scrolling_dialogue = get_node_or_null("ScrollingDialogue") as ScrollingDialogue
	if scrolling_dialogue != null:
		if not scrolling_dialogue.player_actor_changed.is_connected(_on_player_actor_changed):
			scrolling_dialogue.player_actor_changed.connect(_on_player_actor_changed)
		if scrolling_dialogue.player_actor != -1 and journal_log != null:
			journal_log.player_actor = scrolling_dialogue.player_actor

	visible = true

func _find_close_button(root: Node) -> Button:
	if root == null:
		return null
	var direct := root.get_node_or_null("CloseButton") as Button
	if direct != null:
		return direct
	return root.get_node_or_null("ScrollContainer/CloseButton") as Button

func _on_player_actor_changed(actor: int) -> void:
	if journal_log != null:
		journal_log.player_actor = actor

func register_submenu(name: String, node: Control) -> void:
	_submenus[name] = node
	if node != null:
		node.visible = false

func open_submenu(name: String) -> void:
	close_submenu()
	var node: Control = _submenus.get(name)
	if node == null:
		match name:
			"journal", "Journal":
				node = journal_overlay
	if node != null:
		node.visible = true
		_active_submenu = name
		_sync_journal_button()

func close_submenu() -> void:
	if _active_submenu != "":
		var node: Control = _submenus.get(_active_submenu)
		if node == null and (_active_submenu == "journal" or _active_submenu == "Journal"):
			node = journal_overlay
		if node != null:
			node.visible = false
	_active_submenu = ""
	if journal_overlay != null:
		journal_overlay.visible = false
	_sync_journal_button()

func toggle_submenu(name: String) -> void:
	if _active_submenu == name:
		close_submenu()
	else:
		open_submenu(name)

func open_journal() -> void:
	open_submenu("journal")

func close_journal() -> void:
	close_submenu()

func toggle_journal() -> void:
	toggle_submenu("journal")

func is_journal_open() -> bool:
	return journal_overlay != null and journal_overlay.visible

var journal_open: bool:
	get:
		return is_journal_open()

func _on_journal_pressed() -> void:
	if _busy:
		return
	if _toggled_just:
		_toggled_just = false
		return
	_busy = true
	toggle_journal()
	_sync_journal_button()
	_busy = false

func _on_journal_toggled(pressed: bool) -> void:
	if _busy:
		return
	_busy = true
	_toggled_just = true
	call_deferred("_reset_toggled_just")
	if pressed and not is_journal_open():
		open_journal()
	elif (not pressed) and is_journal_open():
		close_journal()
	_sync_journal_button()
	_busy = false

func _reset_toggled_just() -> void:
	_toggled_just = false

func _on_submenu_close_requested() -> void:
	close_submenu()

func _on_overlay_close_requested() -> void:
	# Overlay closed itself (e.g. its own close button); sync button/state.
	if journal_button != null:
		journal_button.set_pressed_no_signal(false)
	close_submenu()

func _sync_journal_button() -> void:
	if journal_button != null:
		journal_button.set_pressed_no_signal(is_journal_open())
