"""Small behavioral checks for replaying Rust-issued render observations in Qt."""

# PIP3 modules
import PySide6.QtWidgets

# local repo modules
import ferrum_qt.canvas.ferrum_render_projection
import ferrum_qt.ferrum.engine as engine
import ferrum_qt.themes.theme_loader


_INTERLEAVED_SOURCE = (
	'<cdml xmlns="urn:ferrum:cdml"><molecule id="m">'
	'<atom id="a" name="C"><point x="0" y="0"/></atom>'
	'<atom id="hidden" name="O" show="no"><point x="20" y="0"/></atom>'
	'<atom id="b" name="N"><point x="40" y="0"/></atom>'
	'</molecule></cdml>'
)


#============================================
def test_qt_replays_painted_batches_around_a_rust_issued_exclusion(
		qapp: PySide6.QtWidgets.QApplication,
		) -> None:
	"""One excluded target stays reported while its neighbors remain selectable."""
	session = engine.DocumentSession.load(_INTERLEAVED_SOURCE)
	snapshot = session.snapshot()
	observation = session.observe_render(snapshot.revision)
	plan = observation.molecule_plans[0].plan
	presentation = session.observe_presentation_render_plan_v1(
		snapshot.revision, snapshot.digest,
	)
	projection = ferrum_qt.canvas.ferrum_render_projection.build_render_projection(
		observation,
		engine.molecule_label_font(),
		presentation,
		ferrum_qt.themes.theme_loader.get_document_display_palette("light"),
	)
	try:
		assert tuple(issue.paint_order for issue in projection.issues) == (1,)
		assert {
			projection.item_targets[item].document_object_id for item in projection.items
		} == {
			batch.target.document_object_id for batch in plan.batches
		}
	finally:
		projection.dispose()
