#!/usr/bin/env python3
"""
Smoke test for FlowExprTest processor using the example_main runner.
Files are preserved in TMP_DIR after the run for inspection.

Covers:
  - Literal string mapping
  - value(key)  — feature attribute lookup
  - env(key)    — workflow variable lookup
  - Combined expressions (env + value, arithmetic)
"""

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

ENGINE_DIR = Path(__file__).resolve().parents[2]  # .../engine
TMP_DIR = Path("/tmp/flow-expr-test")


def build_workflow(input_path: str, output_path: str) -> dict:
    return {
        "id": "a1b2c3d4-e5f6-0001-abcd-ef1234567890",
        "name": "FlowExprTest smoke test",
        "entryGraphId": "b2c3d4e5-f6a7-0001-bcde-f12345678901",
        "with": {
            "prefix": "hello",
        },
        "graphs": [
            {
                "id": "b2c3d4e5-f6a7-0001-bcde-f12345678901",
                "name": "main",
                "nodes": [
                    {
                        "id": "11111111-1111-1111-1111-111111111111",
                        "name": "Reader",
                        "type": "action",
                        "action": "JsonReader",
                        "with": {
                            "dataset": f'"{input_path}"',
                        },
                    },
                    {
                        "id": "22222222-2222-2222-2222-222222222222",
                        "name": "FlowExprTest",
                        "type": "action",
                        "action": "FlowExprTest",
                        "with": {
                            "mappings": [
                                # literal string — no expression engine involved
                                {
                                    "attribute": "label",
                                    "value": {
                                        "type": "string",
                                        "value": "flow-expr-test",
                                    },
                                },
                                # value(key) — reads from feature attributes
                                {
                                    "attribute": "name_copy",
                                    "value": {
                                        "type": "expr",
                                        "value": 'value("name")',
                                    },
                                },
                                # env(key) — reads from workflow with-variables
                                {
                                    "attribute": "prefix",
                                    "value": {
                                        "type": "expr",
                                        "value": 'env("prefix")',
                                    },
                                },
                                # combined: env + value
                                {
                                    "attribute": "greeting",
                                    "value": {
                                        "type": "expr",
                                        "value": 'env("prefix") + ", " + value("name")',
                                    },
                                },
                                # arithmetic on feature attribute
                                {
                                    "attribute": "score_plus_one",
                                    "value": {
                                        "type": "expr",
                                        "value": 'value("score") + 1',
                                    },
                                },
                            ]
                        },
                    },
                    {
                        "id": "33333333-3333-3333-3333-333333333333",
                        "name": "Writer",
                        "type": "action",
                        "action": "FeatureWriter",
                        "with": {
                            "format": "json",
                            "output": f'"{output_path}"',
                        },
                    },
                ],
                "edges": [
                    {
                        "id": "e0000001-0000-0000-0000-000000000001",
                        "from": "11111111-1111-1111-1111-111111111111",
                        "to": "22222222-2222-2222-2222-222222222222",
                        "fromPort": "default",
                        "toPort": "default",
                    },
                    {
                        "id": "e0000002-0000-0000-0000-000000000002",
                        "from": "22222222-2222-2222-2222-222222222222",
                        "to": "33333333-3333-3333-3333-333333333333",
                        "fromPort": "default",
                        "toPort": "default",
                    },
                ],
            }
        ],
    }


if TMP_DIR.exists():
    shutil.rmtree(TMP_DIR)
TMP_DIR.mkdir(parents=True)

input_path = TMP_DIR / "input.json"
output_path = TMP_DIR / "output.json"
workflow_path = TMP_DIR / "workflow.yml"

input_data = [
    {"name": "Alice", "score": 42},
    {"name": "Bob", "score": 7},
]
input_path.write_text(json.dumps(input_data))
workflow_path.write_text(json.dumps(build_workflow(str(input_path), str(output_path))))

env = os.environ.copy()
env["FLOW_EXAMPLE_TARGET_WORKFLOW"] = str(workflow_path)
env["RUST_LOG"] = "error"

result = subprocess.run(
    ["cargo", "run", "--package", "reearth-flow-examples", "--example", "example_main"],
    cwd=ENGINE_DIR,
    env=env,
    text=True,
)

if result.returncode != 0:
    sys.exit(f"Binary exited with code {result.returncode}")

assert output_path.exists(), f"Output file not created: {output_path}"
output = json.loads(output_path.read_text())

expected = [
    {
        "name": "Alice",
        "score": 42,
        "label": "flow-expr-test",
        "name_copy": "Alice",
        "prefix": "hello",
        "greeting": "hello, Alice",
        "score_plus_one": 43,
    },
    {
        "name": "Bob",
        "score": 7,
        "label": "flow-expr-test",
        "name_copy": "Bob",
        "prefix": "hello",
        "greeting": "hello, Bob",
        "score_plus_one": 8,
    },
]
assert output == expected, f"Output mismatch:\n  got:      {json.dumps(output, indent=2)}\n  expected: {json.dumps(expected, indent=2)}"

print("OK")
