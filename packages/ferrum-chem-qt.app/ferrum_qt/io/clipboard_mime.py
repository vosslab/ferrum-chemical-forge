"""Shared ownership marker for Ferrum-created clipboard MIME data."""

# Dynamic QObject property used to identify Python-wrapped MIME objects which
# must be released before interpreter shutdown. The value remains compatible with
# existing clipboard observers; the Python constant carries current product naming.
FERRUM_OWNED_MIME_PROPERTY = "bkchem_qt_owned_mime_data"
