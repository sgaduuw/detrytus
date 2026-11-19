"""Test marks for the optional Rust backend.

Tests that need the detrytus-rs package skip without it, unless
DETRYTUS_REQUIRE_RUST is set, as in CI's bindings job, where a missing
package must fail rather than pass by skipping.
"""

import os

import pytest

from detrytus import rust

needs_rust = pytest.mark.skipif(
    not rust.available() and not os.environ.get("DETRYTUS_REQUIRE_RUST"),
    reason="detrytus-rs is not installed",
)
