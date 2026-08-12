#!/bin/sh

source source_me.sh
pytest tests/
pytest packages/ferrum-rust/crates/api/tests
pytest packages/ferrum-chem-qt.app/tests
