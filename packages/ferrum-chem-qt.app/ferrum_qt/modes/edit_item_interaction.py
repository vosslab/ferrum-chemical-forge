"""Route Edit-mode double-clicks from projected items to public edit actions."""

# Standard Library
import collections.abc

# local repo modules
import ferrum_qt.actions.object_actions
import ferrum_qt.canvas.items.atom_item
import ferrum_qt.canvas.items.bond_item
import ferrum_qt.canvas.items.text_item


#============================================
def open_item_editor(
		item: object,
		atom_editor: collections.abc.Callable[[object], None],
		bond_editor: collections.abc.Callable[[object], None],
		scene: object | None,
		window: object | None,
		) -> None:
	"""Route one projected item to its detached public editing action.

	Atoms and bonds use the mode's session-bound property editors.  Text first
	replaces scene selection, then asks the public action to resolve the current
	durable selection.  The modal action therefore never retains a disposable Qt
	item across a backend reprojection.
	"""
	if isinstance(item, ferrum_qt.canvas.items.atom_item.AtomItem):
		atom_editor(item)
		return
	if isinstance(item, ferrum_qt.canvas.items.bond_item.BondItem):
		bond_editor(item)
		return
	if not isinstance(item, ferrum_qt.canvas.items.text_item.TextItem):
		return
	if scene is None or window is None:
		return
	scene.clearSelection()
	item.setSelected(True)
	ferrum_qt.actions.object_actions.edit_selected_text(window)
