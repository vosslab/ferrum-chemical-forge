"""Portable action-ID registry bound to Ferrum-owned Qt actions."""

# Standard Library
import dataclasses
import functools
import re

# PIP3 modules
import PySide6.QtGui
import PySide6.QtWidgets


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class MenuAction:
	"""One portable command declaration shared with menu and keybinding clients."""

	id: str
	label_key: str
	help_key: str
	accelerator: str | None
	handler: object
	enabled_when: object
	shortcut_exemption_reason: str | None = None
	lifecycle: str = "static"

	@property
	def label(self) -> str:
		"""Return the untranslated source label used by the Qt action."""
		return self.label_key

	@property
	def help_text(self) -> str:
		"""Return the untranslated source help text."""
		return self.help_key


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class LiveActionView:
	"""One immutable palette-facing snapshot of a registered live QAction."""

	action_id: str
	label: str
	help_text: str
	qt_action: PySide6.QtGui.QAction
	enabled: bool


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class _QtActionBinding:
	"""Retain one live Qt client and its guarded retirement ownership."""

	qt_action: PySide6.QtGui.QAction
	binding_token: int
	declaration_owned_by_binding: bool


#============================================
class ActionRegistry:
	"""Store portable action declarations and their live Qt clients."""

	#============================================
	def __init__(self) -> None:
		"""Create an empty registry."""
		self._actions: dict[str, MenuAction] = {}
		self._qt_actions: dict[str, _QtActionBinding] = {}
		self._action_ids_by_identity: dict[int, str] = {}
		self._next_binding_token = 1
		self._dynamic_lifecycles: dict[str, str] = {}
		self._dynamic_menus: dict[str, PySide6.QtWidgets.QMenu] = {}
		self._dynamic_menu_ids_by_identity: dict[int, str] = {}

	#============================================
	def _validate_action_id(self, action_id: str) -> None:
		"""Require one stable lower-case dotted action or menu identity."""
		if type(action_id) is not str or not re.fullmatch(
				r"[a-z][a-z0-9_]*(?:\.[a-z][a-z0-9_]*)*", action_id,
				):
			raise ValueError(f"Invalid Ferrum action ID: '{action_id}'")

	#============================================
	def _bind_existing_action(
			self, action_id: str, qt_action: PySide6.QtGui.QAction,
			*, declaration_owned_by_binding: bool,
			) -> None:
		"""Bind an existing action without changing any feature-owned Qt state."""
		if not isinstance(qt_action, PySide6.QtGui.QAction):
			raise TypeError("Ferrum action registrations require a QAction.")
		if action_id in self._qt_actions:
			raise ValueError(f"Duplicate Qt action binding: '{action_id}'")
		identity = id(qt_action)
		if identity in self._action_ids_by_identity:
			existing_id = self._action_ids_by_identity[identity]
			raise ValueError(
				f"QAction identity is already registered as '{existing_id}'.",
			)
		binding_token = self._next_binding_token
		self._next_binding_token += 1
		self._qt_actions[action_id] = _QtActionBinding(
			qt_action, binding_token, declaration_owned_by_binding,
		)
		self._action_ids_by_identity[identity] = action_id
		# Capture only immutable ID/token data so a late callback cannot retire a
		# replacement binding that later reuses this stable action ID.
		qt_action.destroyed.connect(functools.partial(
			self._retire_destroyed_qt_action, action_id, binding_token,
		))

	#============================================
	def _retire_destroyed_qt_action(
			self, action_id: str, binding_token: int, *_signal_args: object,
			) -> None:
		"""Remove exactly the binding whose feature-owned QObject was destroyed."""
		binding = self._qt_actions.get(action_id)
		if binding is None or binding.binding_token != binding_token:
			return
		self._qt_actions.pop(action_id)
		identity = id(binding.qt_action)
		if self._action_ids_by_identity.get(identity) == action_id:
			self._action_ids_by_identity.pop(identity)
		if binding.declaration_owned_by_binding:
			self._actions.pop(action_id, None)

	#============================================
	def register(self, action: MenuAction) -> None:
		"""Register one declaration, rejecting duplicate action IDs."""
		self._validate_action_id(action.id)
		if action.id in self._actions:
			raise ValueError(f"Duplicate action ID: '{action.id}'")
		if not action.label.strip() or not action.help_text.strip():
			raise ValueError(f"Ferrum action '{action.id}' needs text and help.")
		if action.accelerator is None and not action.shortcut_exemption_reason:
			raise ValueError(
			f"Ferrum action '{action.id}' needs a shortcut or exemption reason.",
		)
		self._actions[action.id] = action

	#============================================
	def register_existing(
			self, action_id: str, qt_action: PySide6.QtGui.QAction, *,
			lifecycle: str = "static", shortcut_exemption_reason: str | None = None,
			) -> None:
		"""Register one feature-owned QAction without replacing its identity or state."""
		self._validate_action_id(action_id)
		if action_id in self._actions:
			raise ValueError(f"Duplicate action ID: '{action_id}'")
		if not isinstance(qt_action, PySide6.QtGui.QAction):
			raise TypeError("Ferrum action registrations require a QAction.")
		label = qt_action.text().replace("&", "").strip()
		help_text = next(
			(text.strip() for text in (
				qt_action.toolTip(), qt_action.statusTip(), qt_action.whatsThis(),
			) if text.strip()),
			"",
		)
		if not label or not help_text:
			raise ValueError(
				f"Ferrum action '{action_id}' needs existing visible text and help text.",
			)
		accelerator = qt_action.shortcut().toString()
		if not accelerator and not shortcut_exemption_reason:
			raise ValueError(
				f"Ferrum action '{action_id}' needs a shortcut or exemption reason.",
			)
		declaration = MenuAction(
			action_id, label, help_text, accelerator or None, qt_action.trigger,
			qt_action.isEnabled, shortcut_exemption_reason, lifecycle,
		)
		self._bind_existing_action(
			action_id, qt_action, declaration_owned_by_binding=True,
		)
		self._actions[action_id] = declaration

	#============================================
	def bind_qt_action(
			self, action_id: str, qt_action: PySide6.QtGui.QAction,
			) -> None:
		"""Bind a predeclared ID to an existing action without changing Qt state."""
		if action_id not in self._actions:
			raise KeyError(action_id)
		self._bind_existing_action(
			action_id, qt_action, declaration_owned_by_binding=False,
		)

	#============================================
	def declare_dynamic_lifecycle(self, owner_id: str, reason: str) -> None:
		"""Record why one feature creates ephemeral actions at runtime.

		Dynamic entries are not retained in the static registry because their
		labels and callbacks are deliberately rebuilt from current presentation
		state.  Their owner must nevertheless make that lifecycle auditable.
		"""
		if not owner_id or not reason.strip():
			raise ValueError("Dynamic action lifecycles need an ID and reason.")
		if owner_id in self._dynamic_lifecycles:
			raise ValueError(f"Duplicate dynamic action lifecycle: '{owner_id}'.")
		self._dynamic_lifecycles[owner_id] = reason

	#============================================
	def register_dynamic_menu(
			self, menu_id: str, menu: PySide6.QtWidgets.QMenu, reason: str,
			) -> None:
		"""Register one feature-owned changing submenu at its YAML placement."""
		self._validate_action_id(menu_id)
		if not isinstance(menu, PySide6.QtWidgets.QMenu):
			raise TypeError("Dynamic menu registrations require a QMenu.")
		if not reason.strip():
			raise ValueError("Dynamic menu registrations need a lifecycle reason.")
		if menu_id in self._dynamic_menus:
			raise ValueError(f"Duplicate dynamic menu ID: '{menu_id}'")
		existing_reason = self._dynamic_lifecycles.get(menu_id)
		if existing_reason is not None and existing_reason != reason:
			raise ValueError(
				f"Dynamic menu '{menu_id}' has conflicting lifecycle reasons.",
			)
		identity = id(menu)
		if identity in self._dynamic_menu_ids_by_identity:
			existing_id = self._dynamic_menu_ids_by_identity[identity]
			raise ValueError(
				f"QMenu identity is already registered as '{existing_id}'.",
			)
		self._dynamic_menus[menu_id] = menu
		self._dynamic_menu_ids_by_identity[identity] = menu_id
		if existing_reason is None:
			self._dynamic_lifecycles[menu_id] = reason

	#============================================
	def dynamic_lifecycles(self) -> dict[str, str]:
		"""Return declared lifecycle reasons for ephemeral action families."""
		return dict(self._dynamic_lifecycles)

	#============================================
	def dynamic_menu_ids(self) -> frozenset[str]:
		"""Return registered state-derived menu identities."""
		return frozenset(self._dynamic_menus)

	#============================================
	def get_dynamic_menu(self, menu_id: str) -> PySide6.QtWidgets.QMenu | None:
		"""Return the registered feature-owned changing submenu when available."""
		return self._dynamic_menus.get(menu_id)

	#============================================
	def get(self, action_id: str) -> MenuAction:
		"""Return one declaration by stable dotted ID."""
		return self._actions[action_id]

	#============================================
	def get_qt_action(self, action_id: str) -> PySide6.QtGui.QAction | None:
		"""Return the live Qt client for an ID when the window supplies one."""
		binding = self._qt_actions.get(action_id)
		return binding.qt_action if binding is not None else None

	#============================================
	def __contains__(self, action_id: str) -> bool:
		"""Return whether the dotted ID is declared."""
		return action_id in self._actions

	#============================================
	def all_actions(self) -> dict[str, MenuAction]:
		"""Return a shallow declaration snapshot."""
		return dict(self._actions)

	#============================================
	def live_action_views(self) -> tuple[LiveActionView, ...]:
		"""Return palette-ready live actions in stable user-facing order.

		The frozen records retain the exact feature-owned QAction while recording
		its current enabled state for presentation.  Callers must recheck that
		live action immediately before invoking it because enablement is dynamic.
		"""
		views = tuple(
			LiveActionView(
				action_id=action_id,
				label=self._actions[action_id].label,
				help_text=self._actions[action_id].help_text,
				qt_action=qt_action,
				enabled=qt_action.isEnabled(),
			)
			for action_id, binding in self._qt_actions.items()
			for qt_action in (binding.qt_action,)
		)
		return tuple(sorted(
			views, key=lambda view: (view.label.casefold(), view.action_id),
		))

	#============================================
	def is_enabled(self, action_id: str, context: object) -> bool:
		"""Evaluate one declaration's optional enablement predicate."""
		predicate = self._actions[action_id].enabled_when
		if predicate is None:
			return True
		if callable(predicate):
			return bool(predicate())
		return bool(getattr(context, predicate, False))
