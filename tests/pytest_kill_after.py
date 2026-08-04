"""Repository-local pytest deadline plugin for hung Qt test processes."""

# Standard Library
import faulthandler
import os
import sys
import threading


_TIMERS: dict[int, threading.Timer] = {}


#============================================
def pytest_addoption(parser: object) -> None:
	"""Add an opt-in wall-clock deadline to pytest's terminal options."""
	group = parser.getgroup("terminal reporting")
	group.addoption(
		"--kill-after",
		action="store",
		type=float,
		default=0.0,
		metavar="SECONDS",
		help="Exit after this many wall-clock seconds; disabled by default.",
	)


#============================================
def _stop_deadline(config: object) -> None:
	"""Cancel the deadline timer associated with one pytest configuration."""
	timer = _TIMERS.pop(id(config), None)
	if timer is not None:
		timer.cancel()


#============================================
def _deadline_reached(seconds: float) -> None:
	"""Report live thread stacks, then exit before a hung teardown can persist."""
	message = f"\n--kill-after deadline reached after {seconds:.3f} seconds\n"
	sys.stderr.write(message)
	sys.stderr.flush()
	faulthandler.dump_traceback(file=sys.stderr, all_threads=True)
	os._exit(124)


#============================================
def pytest_configure(config: object) -> None:
	"""Start the requested deadline after pytest has parsed its options."""
	seconds = float(config.getoption("kill_after"))
	if seconds <= 0.0:
		return
	timer = threading.Timer(seconds, _deadline_reached, args=(seconds,))
	timer.daemon = True
	_TIMERS[id(config)] = timer
	timer.start()


#============================================
def pytest_sessionfinish(session: object, exitstatus: object) -> None:
	"""Cancel the deadline after tests and ordinary fixture teardown finish."""
	_stop_deadline(session.config)


#============================================
def pytest_unconfigure(config: object) -> None:
	"""Cancel the deadline during ordinary or early pytest shutdown."""
	_stop_deadline(config)
