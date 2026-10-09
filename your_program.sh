#!/bin/sh
#
# Use this script to run your program LOCALLY.
#
# Note: Changing this script WILL NOT affect how CodeCrafters runs your program.
#
# Learn more: https://codecrafters.io/program-interface

set -e # Exit early if any commands fail

# Copied from .codecrafters/compile.sh
#
# - Edit this to change how your program compiles locally
# - Edit .codecrafters/compile.sh to change how your program compiles remotely
(
  cd "$(dirname "$0")" # Ensure compile steps are run within the repository directory
  cargo build --release --target-dir=/tmp/codecrafters-build-claude-code-rust --manifest-path Cargo.toml
  # cargo build --target-dir=/tmp/codecrafters-build-claude-code-rust --manifest-path Cargo.toml
)

# Copied from .codecrafters/run.sh
#
# - Edit this to change how your program runs locally
# - Edit .codecrafters/run.sh to change how your program runs remotely
exec /tmp/codecrafters-build-claude-code-rust/release/codecrafters-claude-code "$@"
# OPENROUTER_API_KEY=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 exec /tmp/codecrafters-build-claude-code-rust/debug/codecrafters-claude-code "$@"
