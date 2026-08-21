"""Prove an isolated installed Ferrum wheel owns reaction aggregate translation."""

# Standard Library
import argparse
import json
import os
import pathlib
import subprocess
import sys
import tempfile


SOURCE = (
	'<cdml><molecule id="left"><atom id="left-a" name="C">'
	'<point x="0" y="0"/></atom></molecule>'
	'<molecule id="right"><atom id="right-a" name="O">'
	'<point x="100" y="0"/></atom></molecule>'
	'<arrow id="arrow"><point x="25" y="0"/><point x="75" y="0"/></arrow>'
	'<reaction id="strict"><reactant idref="left"/><product idref="right"/>'
	'<arrow idref="arrow"/></reaction></cdml>'
)


#============================================
class ReactionTranslationE2eError(RuntimeError):
	"""Raised when installed-wheel reaction translation loses durable truth."""


#============================================
def _run(*command: str, environment: dict[str, str]) -> str:
	"""Run one isolated subprocess and return its standard output."""
	result = subprocess.run(
		command, env=environment, text=True, stdout=subprocess.PIPE,
		stderr=subprocess.PIPE, check=False,
	)
	if result.returncode:
		raise ReactionTranslationE2eError(
			"command failed (%d): %s\n%s" % (
				result.returncode, " ".join(command), result.stderr.strip(),
			),
		)
	return result.stdout


#============================================
def _selection(session: object) -> object:
	"""Return one exact installed Rust selection without Python-side membership inference."""
	snapshot = session.snapshot()
	observation = session.observe_reaction_list_v1(snapshot.revision, snapshot.digest)
	return session.select_reaction_v1(observation, "strict")


#============================================
def _probe() -> dict[str, object]:
	"""Exercise the public opaque aggregate gesture from an installed root extension."""
	import ferrum_chem

	extension_path = pathlib.Path(ferrum_chem.__file__).resolve()
	if pathlib.Path(sys.prefix).resolve() not in extension_path.parents:
		raise ReactionTranslationE2eError(
			"Ferrum chemistry did not load from the isolated wheel environment",
		)
	session = ferrum_chem.DocumentSession.load(SOURCE)
	gesture = session.begin_reaction_translation_v1(_selection(session), 0.0, 0.0)
	preview = session.preview_reaction_translation_v1(gesture, 20.0, 10.0)
	prepared = session.prepare_reaction_translation_v1(gesture, preview)
	accepted = session.commit_reaction_translation_v1(prepared)
	changed = accepted.result.observation.snapshot
	if changed.revision != 1 or '<reaction id="strict"' not in changed.cdml:
		raise ReactionTranslationE2eError("installed gesture did not commit one reaction transition")
	if not all(reference in changed.cdml for reference in (
		'idref="left"', 'idref="right"', 'idref="arrow"',
	)):
		raise ReactionTranslationE2eError("installed gesture rewrote reaction member references")
	if '<point x="0.706cm" y="0.353cm"' not in changed.cdml:
		raise ReactionTranslationE2eError("installed gesture did not move the aggregate")
	if session.undo(changed.revision).observation.snapshot.cdml != SOURCE:
		raise ReactionTranslationE2eError("installed gesture undo did not restore the source")
	return {
		"schema": "ferrum-reaction-translation-wheel-e2e-v1",
		"reaction_id": accepted.reaction_id,
		"root_extension": extension_path.name,
	}


#============================================
def main() -> int:
	"""Install one wheel into an isolated venv and execute the opaque public path."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--wheel", type=pathlib.Path)
	parser.add_argument("--probe", action="store_true")
	arguments = parser.parse_args()
	if arguments.probe:
		print(json.dumps(_probe(), sort_keys=True))
		return 0
	if arguments.wheel is None or not arguments.wheel.is_file():
		raise ReactionTranslationE2eError("--wheel must name one direct wheel artifact")
	environment = os.environ.copy()
	environment["PYTHONDONTWRITEBYTECODE"] = "1"
	with tempfile.TemporaryDirectory(prefix="ferrum-reaction-translation-wheel-") as directory:
		venv = pathlib.Path(directory) / "venv"
		_run(sys.executable, "-B", "-m", "venv", str(venv), environment=environment)
		python = venv / "bin" / "python"
		_run(str(python), "-B", "-m", "pip", "install", "--no-deps", str(arguments.wheel.resolve()), environment=environment)
		output = _run(str(python), "-I", "-B", str(pathlib.Path(__file__).resolve()), "--probe", environment=environment)
	value = json.loads(output)
	if value["reaction_id"] != "strict":
		raise ReactionTranslationE2eError("installed translation returned the wrong reaction")
	print(json.dumps(value, sort_keys=True))
	return 0


if __name__ == "__main__":
	main()
