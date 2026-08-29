"""Prove a Rust-issued E/Z carrier mark reaches the visible Qt projection."""

# Standard Library
import json

# local repo modules
import ferrum_qt_e2e


ferrum_qt_e2e.select_offscreen_qt_platform()

# PIP3 modules
import PySide6.QtCore
import PySide6.QtWidgets

# local repo modules
import ferrum_chem
import ferrum_qt.canvas.items.ferrum_plan_item
import ferrum_qt.ferrum.document_tab
import ferrum_qt.ferrum.main_window
import ferrum_qt.main_window
import ferrum_qt.themes.theme_manager


#============================================
class EzCarrierMarkProjectionE2eError(RuntimeError):
	"""Report one broken Rust-to-Qt E/Z carrier-mark delivery boundary."""


#============================================
def _molecule_report(observation: object, molecule_id: str) -> dict:
	"""Request the public Rust report for the exact installed observation."""
	response = json.loads(ferrum_chem.execute_operation_v1(json.dumps({
		"schema": "ferrum-operation-request-v1",
		"request_id": "ez-carrier-mark-projection",
		"operation": {
			"kind": "document.molecule.report.v1",
			"snapshot": {
				"cdml": observation.snapshot.cdml,
				"revision": observation.snapshot.revision,
				"digest_hex": observation.snapshot.digest,
			},
			"molecule_ids": [molecule_id],
		},
	})))
	record = response["outcome"]["report"]["records"][0]
	return record


#============================================
def _carrier_geometry(render: object) -> tuple[float, float]:
	"""Return the midpoint of Rust's one explicit E/Z carrier-mark operation."""
	for plan_entry in render.molecule_plans:
		for batch in plan_entry.plan.batches:
			content = batch.content
			if type(content) is not ferrum_chem.BondRenderBatchV1:
				continue
			for operation in content.typed_operations:
				if operation.kind == "double_bond_carrier_mark":
					payload = operation.operation
					return (
						(payload.start.x + payload.end.x) / 2.0,
						(payload.start.y + payload.end.y) / 2.0,
					)
	raise EzCarrierMarkProjectionE2eError(
		"Rust render binding omitted the E/Z double_bond_carrier_mark operation",
	)


#============================================
def _scene_receives_carrier_geometry(
		tab: ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab,
		carrier_point: PySide6.QtCore.QPointF,
		) -> bool:
	"""Require the visible scene to hit-test the exact Rust-issued mark midpoint."""
	for item in tab.view.scene().items(carrier_point):
		if isinstance(item, ferrum_qt.canvas.items.ferrum_plan_item.FerrumPlanItem):
			local_point = item.mapFromScene(carrier_point)
			if item.shape().contains(local_point):
				return True
	return False


#============================================
def main() -> int:
	"""Create native E/Z chemistry and follow its exact rendering into real Qt."""
	app = PySide6.QtWidgets.QApplication.instance() or PySide6.QtWidgets.QApplication([])
	placement = ferrum_chem.validate_insertion_placement_v1(40.0, 200.0, 150.0)
	session = ferrum_chem.DocumentSession.load("<cdml xmlns='urn:ferrum:cdml'/>")
	molecule = ferrum_chem.prepare_smiles_molecule_v1("F/C=C/F", placement)
	committed = session.apply_document_operation_v1(
		0, ferrum_chem.DocumentOperationV1.insert_molecule_v1(molecule),
	)
	observation = committed.observation
	molecule_id = observation.projection.molecules[0].document_object_id
	record = _molecule_report(observation, molecule_id)
	if not record["stereo_depiction"]["double_bond_carrier_marks"]:
		raise EzCarrierMarkProjectionE2eError(
			"Rust molecule report omitted the E/Z carrier-mark receipt",
		)
	carrier_x, carrier_y = _carrier_geometry(
		session.observe_render(observation.snapshot.revision),
	)
	theme_manager = ferrum_qt.themes.theme_manager.ThemeManager(app)
	window = ferrum_qt.main_window.MainWindow(theme_manager)
	tab = ferrum_qt.ferrum.document_tab.FerrumNativeDocumentTab(
		observation.snapshot.cdml, "native-ez.cdml",
		window._require_document_display_palette(),
	)
	window._register_native_tab(tab, activate=True)
	window.show()
	app.processEvents()
	try:
		carrier_point = PySide6.QtCore.QPointF(carrier_x, carrier_y)
		if not _scene_receives_carrier_geometry(tab, carrier_point):
			raise EzCarrierMarkProjectionE2eError(
				"visible Qt scene did not receive Rust E/Z carrier-mark geometry",
			)
		print(json.dumps({"schema": "ferrum-ez-carrier-mark-projection-e2e-v1", "status": "ok"}))
		return 0
	finally:
		ferrum_qt_e2e.close_e2e_main_window(window, app)


if __name__ == "__main__":
	raise SystemExit(main())
