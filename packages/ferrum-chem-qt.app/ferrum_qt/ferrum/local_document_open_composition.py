"""Compose the explicit window callbacks used by Local Document Open."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.ferrum.local_document_open_contract
import ferrum_qt.ferrum.canvas_interaction
import ferrum_qt.ferrum.tab_operations


#============================================
def compose_local_document_open_host(
		window: PySide6.QtWidgets.QMainWindow,
		) -> ferrum_qt.ferrum.local_document_open_contract.LocalDocumentOpenHost:
	"""Bind the controller only to deliberate window capabilities."""
	return ferrum_qt.ferrum.local_document_open_contract.LocalDocumentOpenHost(
		parent=window,
		translate=window.tr,
		register_action=lambda action_id, action, lifecycle: window._register_action(
			action_id, action, lifecycle=lifecycle,
		),
		action_refresh=window._refresh_actions,
		active_tab=window._active_native_tab,
		tab_is_registered=lambda tab: window._native_tabs_by_page.get(tab) is tab,
		tab_widget_current=lambda: window._tab_widget.currentWidget(),
		tab_widget_index=lambda tab: window._tab_widget.indexOf(tab),
		tab_widget_set_current_index=lambda index: window._tab_widget.setCurrentIndex(index),
		publish_open_tab=window._publish_local_open_tab,
		finish_open_publication=window._finish_local_open_publication,
		commit_open_replacement=window._commit_local_open_replacement,
		finish_open_replacement=window._finish_local_open_replacement,
		palette=lambda: window._require_document_display_palette(),
		present_refusal=lambda request: window._show_edit_refusal(request),
		show_status=lambda message, timeout: window.statusBar().showMessage(message, timeout),
		snapshot_busy=window._snapshot_export_busy,
		shutdown_prepared=lambda: window._shutdown_prepared,
		tab_has_active_canvas_interaction=lambda tab: ferrum_qt.ferrum.canvas_interaction.
			tab_has_active_native_canvas_interaction(window, tab),
		cancel_active_pointer_authoring=window.cancel_active_pointer_authoring,
		tab_has_active_operation=lambda tab: ferrum_qt.ferrum.tab_operations.
			tab_has_active_native_operation(window, tab),
		tab_has_conflict_except_lease=lambda tab, lease: ferrum_qt.ferrum.tab_operations.
			tab_has_active_native_operation_except_lease(window, tab, lease),
		native_tab_for_origin_token=lambda token: next((
			tab for tab in window._native_tabs_by_page.values()
			if tab.local_document_origin_token == token
		), None),
		prompt_native_save=window._prompt_native_save,
		save_native_tab_to_path=window._save_native_tab_to_path,
		record_recent_success=lambda path: window._native_recent_files.record_confirmed_path(path),
		handle_recent_failure=lambda path, failure: window._native_recent_files.
			handle_failed_recent_open(path, failure),
		emit_completed=window.local_document_open_completed.emit,
		emit_queue_drained=window.local_document_open_queue_drained.emit,
	)
