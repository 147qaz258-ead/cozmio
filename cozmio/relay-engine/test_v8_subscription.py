#!/usr/bin/env python3
"""Compatibility entry point for the real Relay subscription verification.

The previous V8-only script accepted a completion-only event and could miss
pre-subscription progress. The real check now lives in test_real_chain.py and
covers replayed buffered progress, live progress, and terminal events.
"""

from test_real_chain import main


if __name__ == "__main__":
    raise SystemExit(main())
