"""Stable CDML I/O facade.

The public API remains unchanged while XML parsing, hydration, and disposable
Qt projection staging remain independently replaceable backend seams.
"""

# local repo modules
import ferrum_qt.io.cdml_document_hydration
import ferrum_qt.io.cdml_projection_staging


PreparedProjection = ferrum_qt.io.cdml_projection_staging.PreparedProjection
decode_compatibility_cdml_file = ferrum_qt.io.cdml_document_hydration.decode_compatibility_cdml_file
decode_compatibility_cdml_string = ferrum_qt.io.cdml_document_hydration.decode_compatibility_cdml_string
dispose_prepared_projection = ferrum_qt.io.cdml_projection_staging.dispose_prepared_projection
hydrate_synchronized_cdml_document = ferrum_qt.io.cdml_document_hydration.hydrate_synchronized_cdml_document
prepare_compatibility_projection_from_cdml = ferrum_qt.io.cdml_projection_staging.prepare_compatibility_projection_from_cdml
prepare_synchronized_projection = ferrum_qt.io.cdml_projection_staging.prepare_synchronized_projection
