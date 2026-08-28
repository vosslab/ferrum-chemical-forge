"""Accessible projection of one immutable Rust template-catalog snapshot."""

import PySide6.QtCore
import PySide6.QtWidgets


class FerrumTemplateCatalogDialog(PySide6.QtWidgets.QDialog):
	"""Browse Rust-issued catalog facts and return one opaque entry key."""

	def __init__(self, parent: PySide6.QtWidgets.QWidget, snapshot: object | None) -> None:
		super().__init__(parent)
		self._snapshot = snapshot
		self._entries: tuple[object, ...] = () if snapshot is None else tuple(snapshot.entries)
		self.setModal(False)
		self.setWindowTitle(self.tr("Template Catalog"))
		self.setAccessibleName(self.tr("Template Catalog"))
		self.setAccessibleDescription(self.tr(
			"Browse built-in and saved Ferrum templates, then place one on the canvas.",
		))
		self.resize(720, 560)
		self._build_widgets()
		self._populate_facets()
		self._refresh_results()
		self._apply_tab_order()
		self.search.setFocus()

	def _build_widgets(self) -> None:
		layout = PySide6.QtWidgets.QVBoxLayout(self)
		filters = PySide6.QtWidgets.QHBoxLayout()
		self.source_label = PySide6.QtWidgets.QLabel(self.tr("Source:"), self)
		filters.addWidget(self.source_label)
		self.source = PySide6.QtWidgets.QComboBox(self)
		self.source.addItem(self.tr("Built-in"), "shipped")
		self.source.addItem(self.tr("My templates"), "user_directory")
		self.source.setAccessibleName(self.tr("Template source"))
		self.source_label.setBuddy(self.source)
		filters.addWidget(self.source)
		self.family_label = PySide6.QtWidgets.QLabel(self.tr("Family:"), self)
		filters.addWidget(self.family_label)
		self.family = PySide6.QtWidgets.QComboBox(self)
		self.family.setAccessibleName(self.tr("Built-in template family"))
		self.family_label.setBuddy(self.family)
		filters.addWidget(self.family)
		self.category_label = PySide6.QtWidgets.QLabel(self.tr("Category:"), self)
		filters.addWidget(self.category_label)
		self.category = PySide6.QtWidgets.QComboBox(self)
		self.category.setAccessibleName(self.tr("Built-in template category"))
		self.category_label.setBuddy(self.category)
		filters.addWidget(self.category, 1)
		layout.addLayout(filters)
		self.search = PySide6.QtWidgets.QLineEdit(self)
		self.search.setPlaceholderText(self.tr("Search this catalog snapshot"))
		self.search.setAccessibleName(self.tr("Search templates"))
		layout.addWidget(self.search)
		self.state = PySide6.QtWidgets.QLabel(self)
		self.state.setWordWrap(True)
		self.state.setAccessibleName(self.tr("Template catalog status"))
		self.state.setTextInteractionFlags(
			PySide6.QtCore.Qt.TextInteractionFlag.TextSelectableByMouse,
		)
		layout.addWidget(self.state)
		self.refusal_toggle = PySide6.QtWidgets.QToolButton(self)
		self.refusal_toggle.setText(self.tr("Show refresh details"))
		self.refusal_toggle.setCheckable(True)
		self.refusal_toggle.setAccessibleName(self.tr("Show template refresh details"))
		layout.addWidget(self.refusal_toggle)
		self.refusal_details = PySide6.QtWidgets.QPlainTextEdit(self)
		self.refusal_details.setReadOnly(True)
		self.refusal_details.setAccessibleName(self.tr("Template refresh details"))
		self.refusal_details.setVisible(False)
		layout.addWidget(self.refusal_details)
		self.results = PySide6.QtWidgets.QListWidget(self)
		self.results.setAccessibleName(self.tr("Template catalog results"))
		self.results.setSelectionMode(
			PySide6.QtWidgets.QAbstractItemView.SelectionMode.SingleSelection,
		)
		layout.addWidget(self.results, 1)
		self.details = PySide6.QtWidgets.QLabel(self)
		self.details.setWordWrap(True)
		self.details.setAccessibleName(self.tr("Selected template details"))
		self.details.setTextInteractionFlags(
			PySide6.QtCore.Qt.TextInteractionFlag.TextSelectableByKeyboard,
		)
		self.details.setFocusPolicy(PySide6.QtCore.Qt.FocusPolicy.StrongFocus)
		layout.addWidget(self.details)
		buttons = PySide6.QtWidgets.QDialogButtonBox(self)
		self.save_button = buttons.addButton(
			self.tr("Save Current as Template..."),
			PySide6.QtWidgets.QDialogButtonBox.ButtonRole.ActionRole,
		)
		self.refresh_button = buttons.addButton(
			self.tr("Refresh"), PySide6.QtWidgets.QDialogButtonBox.ButtonRole.ActionRole,
		)
		self.place_button = buttons.addButton(
			self.tr("Place on Canvas"),
			PySide6.QtWidgets.QDialogButtonBox.ButtonRole.AcceptRole,
		)
		self.cancel_button = buttons.addButton(
			PySide6.QtWidgets.QDialogButtonBox.StandardButton.Cancel,
		)
		self._configure_buttons(layout, buttons)
		self.source.currentIndexChanged.connect(self._refresh_results)
		self.family.currentIndexChanged.connect(self._refresh_results)
		self.category.currentIndexChanged.connect(self._refresh_results)
		self.search.textChanged.connect(self._refresh_results)
		self.results.currentItemChanged.connect(self._update_details)
		self.results.itemDoubleClicked.connect(self._accept_selected)
		self.refusal_toggle.toggled.connect(self.refusal_details.setVisible)

	def _configure_buttons(
			self, layout: PySide6.QtWidgets.QVBoxLayout,
			buttons: PySide6.QtWidgets.QDialogButtonBox,
			) -> None:
		self.save_button.setAccessibleName(self.tr("Save current document as a template"))
		self.refresh_button.setAccessibleName(self.tr("Refresh template catalog"))
		self.place_button.setAccessibleName(self.tr("Place selected template on canvas"))
		self.cancel_button.setAccessibleName(self.tr("Close template catalog"))
		self.save_button.setDefault(False)
		self.save_button.setAutoDefault(False)
		self.refresh_button.setDefault(False)
		self.refresh_button.setAutoDefault(False)
		self.place_button.setDefault(False)
		self.place_button.setAutoDefault(False)
		self.cancel_button.setAutoDefault(False)
		buttons.accepted.connect(self._accept_selected)
		buttons.rejected.connect(self.reject)
		layout.addWidget(buttons)

	def _apply_tab_order(self) -> None:
		PySide6.QtWidgets.QWidget.setTabOrder(self.source, self.family)
		PySide6.QtWidgets.QWidget.setTabOrder(self.family, self.category)
		PySide6.QtWidgets.QWidget.setTabOrder(self.category, self.search)
		PySide6.QtWidgets.QWidget.setTabOrder(self.search, self.results)
		PySide6.QtWidgets.QWidget.setTabOrder(self.results, self.details)
		PySide6.QtWidgets.QWidget.setTabOrder(self.details, self.refusal_toggle)
		PySide6.QtWidgets.QWidget.setTabOrder(self.refusal_toggle, self.refusal_details)
		PySide6.QtWidgets.QWidget.setTabOrder(self.refusal_details, self.save_button)
		PySide6.QtWidgets.QWidget.setTabOrder(self.save_button, self.refresh_button)
		PySide6.QtWidgets.QWidget.setTabOrder(self.refresh_button, self.place_button)
		PySide6.QtWidgets.QWidget.setTabOrder(self.place_button, self.cancel_button)

	def selected_key(self) -> str | None:
		item = self.results.currentItem()
		return None if item is None else item.data(PySide6.QtCore.Qt.ItemDataRole.UserRole)

	def selected_snapshot(self) -> object | None:
		"""Return the retained native capability for the current admitted selection."""
		return self._snapshot if self.selected_key() is not None else None

	def replace_snapshot(self, snapshot: object) -> None:
		"""Project a fresh snapshot while retaining browse choices when possible."""
		selection = self.selected_key()
		facet_values = (
			self.source.currentData(), self.family.currentData(), self.category.currentData(),
		)
		self._snapshot = snapshot
		self._entries = tuple(snapshot.entries)
		for widget in (self.family, self.category):
			widget.blockSignals(True)
			widget.clear()
		self._populate_facets()
		self.source.setCurrentIndex(max(self.source.findData(facet_values[0]), 0))
		self.family.setCurrentIndex(max(self.family.findData(facet_values[1]), 0))
		self.category.setCurrentIndex(max(self.category.findData(facet_values[2]), 0))
		for widget in (self.family, self.category):
			widget.blockSignals(False)
		self._refresh_results(selection)

	def set_refresh_busy(self, busy: bool) -> None:
		self.refresh_button.setEnabled(not busy)
		if busy:
			self.announce(self.tr("Refreshing the template catalog..."))

	def announce(self, message: str, *, focus_search: bool = False) -> None:
		"""Set visible accessible state and, when requested, restore catalog focus."""
		self.state.setText(message)
		if focus_search:
			self.search.setFocus()

	def report_refresh_complete(self) -> None:
		self.announce(self.tr("Template catalog refreshed. Choose a template."))

	def report_saved_and_refreshed(self) -> None:
		self.announce(self.tr("Template saved and catalog refreshed. Choose a template."))

	def set_unavailable(self, message: str) -> None:
		self.announce(message, focus_search=True)
		self.results.clear()
		self.details.setText(self.tr("Refresh or close this catalog."))
		self.place_button.setEnabled(False)
		self.place_button.setDefault(False)
		self.place_button.setAutoDefault(False)

	def _accept_selected(self, *_unused: object) -> None:
		if self.selected_key() is not None:
			self.accept()

	def _populate_facets(self) -> None:
		self.family.addItem(self.tr("All families"), None)
		self.category.addItem(self.tr("All categories"), None)
		seen_families: set[str] = set()
		seen_categories: set[str] = set()
		for entry in self._entries:
			if entry.source_kind != "shipped":
				continue
			if entry.family is not None and entry.family not in seen_families:
				seen_families.add(entry.family)
				self.family.addItem(entry.family_label, entry.family)
			if entry.category is not None and entry.category not in seen_categories:
				seen_categories.add(entry.category)
				self.category.addItem(entry.category_label, entry.category)

	def _refresh_results(self, selected_key: str | None = None) -> None:
		source = self.source.currentData()
		built_in = source == "shipped"
		for widget in (self.family_label, self.family, self.category_label, self.category):
			widget.setVisible(built_in)
		query = self.search.text().strip().casefold()
		entries = tuple(entry for entry in self._entries if self._entry_matches(
			entry, source, built_in, query,
		))
		self.results.clear()
		for entry in entries:
			item = PySide6.QtWidgets.QListWidgetItem(entry.label)
			item.setData(PySide6.QtCore.Qt.ItemDataRole.UserRole, entry.key)
			item.setToolTip(self._entry_tooltip(entry))
			self.results.addItem(item)
		self._set_result_state(source, query, bool(entries))
		self._set_refusal_details(source)
		index = next((index for index in range(self.results.count()) if (
			self.results.item(index).data(PySide6.QtCore.Qt.ItemDataRole.UserRole) == selected_key
		)), 0)
		if self.results.count():
			self.results.setCurrentRow(index)
		self._update_details()

	def _set_result_state(self, source: str, query: str, has_entries: bool) -> None:
		if has_entries:
			if source == "user_directory" and self._has_refusals():
				self.announce(self.tr(
					"Some neighboring templates need attention; admitted templates remain available.",
				))
			else:
				self.announce(self.tr("Choose a template, then use Place on Canvas."))
			return
		if query:
			self.announce(self.tr(
				"No matches for '{0}'. Clear the search or change the source.",
			).format(self.search.text().strip()))
		elif source == "user_directory":
			self.announce(self.tr(
				"No eligible saved templates yet. Save the current document or refresh.",
			))
		else:
			self.announce(self.tr("No built-in templates are available in this snapshot."))

	def _set_refusal_details(self, source: str) -> None:
		visible = source == "user_directory" and self._has_refusals()
		self.refusal_toggle.setVisible(visible)
		self.refusal_details.setVisible(visible and self.refusal_toggle.isChecked())
		if visible:
			self.refusal_details.setPlainText("\n".join(
				self._refusal_detail(refusal)
				for refusal in self._snapshot.refusals
			))

	def _has_refusals(self) -> bool:
		return self._snapshot is not None and bool(self._snapshot.refusals)

	def _refusal_detail(self, refusal: object) -> str:
		name = refusal.basename or self.tr("Template")
		return self.tr("{0} ({1}): {2}. {3}").format(
			name, refusal.occurrences, self._refusal_category(refusal.category),
			self._refusal_recovery(refusal.recovery),
		)

	def _refusal_category(self, category: str) -> str:
		return {
			"directory_symlink": self.tr("The template directory is a symbolic link"),
			"directory_not_directory": self.tr("The template location is not a directory"),
			"filename_non_utf8": self.tr("A template filename cannot be represented"),
			"candidate_symlink": self.tr("The template is a symbolic link"),
			"candidate_not_regular": self.tr("The template is not a regular file"),
			"candidate_open_failed": self.tr("Ferrum cannot open the template"),
			"candidate_read_failed": self.tr("Ferrum cannot read the template"),
			"file_too_large": self.tr("The template is too large"),
			"catalog_limit_exceeded": self.tr("The catalog reached a safety limit"),
			"utf8_invalid": self.tr("The template text is not valid UTF-8"),
			"document_admission": self.tr("Ferrum cannot admit this document"),
			"duplicate_content": self.tr("The same template content is already listed"),
			"selection_not_found": self.tr("The selected template is unavailable"),
			"selection_snapshot_stale": self.tr("The catalog snapshot is stale"),
			"document_stale": self.tr("The document changed"),
			"renderer_refused": self.tr("Ferrum cannot render this template"),
			"session_conflict": self.tr("Ferrum could not change the document"),
		}.get(category, self.tr("Ferrum could not admit this template"))

	def _refusal_recovery(self, recovery: str) -> str:
		return {
			"refresh": self.tr("Refresh the catalog."),
			"fix_directory": self.tr("Fix the directory, then refresh."),
			"fix_file": self.tr("Fix the file, then refresh."),
			"choose_entry": self.tr("Choose a different template."),
			"document_unchanged": self.tr("Keep the document unchanged, then choose again."),
		}.get(recovery, self.tr("Refresh the catalog."))

	def _entry_matches(self, entry: object, source: str, built_in: bool, query: str) -> bool:
		if entry.source_kind != source:
			return False
		if built_in and self.family.currentData() is not None:
			if entry.family != self.family.currentData():
				return False
		if built_in and self.category.currentData() is not None:
			if entry.category != self.category.currentData():
				return False
		return not query or any(query in term.casefold() for term in entry.search_terms)

	def _entry_tooltip(self, entry: object) -> str:
		return self.tr("{0}; {1}").format(
			entry.provenance_source_id, entry.compatibility_profile,
		)

	def _update_details(self, *_unused: object) -> None:
		entry = next((value for value in self._entries if value.key == self.selected_key()), None)
		admitted = entry is not None
		self.place_button.setEnabled(admitted)
		self.place_button.setDefault(admitted)
		self.place_button.setAutoDefault(admitted)
		if not admitted:
			self.details.setText(self.tr(
				"Select an admitted template to inspect its provenance and compatibility.",
			))
			return
		self.details.setText(self._entry_detail(entry))

	def _entry_detail(self, entry: object) -> str:
		provenance = "{0}; {1}".format(
			entry.provenance_source_kind, entry.provenance_source_id,
		)
		if entry.provenance_license_spdx is not None:
			provenance += "; " + entry.provenance_license_spdx
		if entry.provenance_reviewed_on is not None:
			provenance += "; reviewed " + entry.provenance_reviewed_on
		if entry.provenance_chemistry_scope is not None:
			provenance += "; " + entry.provenance_chemistry_scope
		limits = self.tr(
			"Limits: {0} entries, {1} candidates, {2} refusals, {3} bytes each, {4} total",
		).format(
			self._snapshot.limits_max_entries, self._snapshot.limits_max_candidates,
			self._snapshot.limits_max_refusals, self._snapshot.limits_max_file_bytes,
			self._snapshot.limits_max_total_bytes,
		)
		return self.tr(
			"Provenance: {0}\nIdentity: {1} {2}\nCompatibility: {3} ({4})\n{5}",
		).format(
			provenance, entry.content_identity_algorithm, entry.content_identity,
			entry.compatibility_profile, entry.compatibility_format, limits,
		)
