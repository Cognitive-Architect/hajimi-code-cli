#!/usr/bin/env bash
set -e
cd "$(dirname "$0")/.."
node tests/e2e/phase1-5-regression/full_chain.test.js
