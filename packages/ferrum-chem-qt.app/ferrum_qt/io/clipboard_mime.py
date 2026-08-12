"""Shared ownership marker for Ferrum-created clipboard MIME data."""

# Dynamic QObject property used to identify Python-wrapped MIME objects which
# must be released before interpreter shutdown.
FERRUM_OWNED_MIME_PROPERTY = "ferrum_qt_owned_mime_data"
