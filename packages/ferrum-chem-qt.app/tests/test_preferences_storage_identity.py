"""Compatibility checks for the Ferrum-Qt preference storage identity."""

# local repo modules
import ferrum_qt.config.preferences


#============================================
def test_product_rename_retains_existing_preference_store() -> None:
	"""Ferrum display branding does not orphan settings stored by existing users."""
	preferences = ferrum_qt.config.preferences.Preferences()

	assert preferences._settings.organizationName() == "BKChem"
	assert preferences._settings.applicationName() == "BKChem-Qt"
