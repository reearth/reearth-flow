#!/usr/bin/env bash
set -e

if ! command -v uv &> /dev/null; then
  echo "Error: uv is not installed."
  echo "Install it with: curl -LsSf https://astral.sh/uv/install.sh | sh"
  exit 1
fi

if [ ! -d ".venv" ]; then
  uv venv --python 3.14
  uv pip install -r requirements.txt
fi
