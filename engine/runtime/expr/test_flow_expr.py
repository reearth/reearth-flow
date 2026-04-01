#!/usr/bin/env python3
"""
Smoke test for FlowExprTest processor using the example_main runner.
Files are preserved in TMP_DIR after the run for inspection.
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
                                {
                                    "attribute": "greeting",
                                    "value": {
                                        "type": "expr",
                                        "value": '"hello, " + getattr("name")',
                                    },
                                },
                                {
                                    "attribute": "label",
                                    "value": {
                                        "type": "string",
                                        "value": "flow-expr-test",
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
    {"name": "Alice", "score": 42, "greeting": "hello, Alice", "label": "flow-expr-test"},
    {"name": "Bob", "score": 7, "greeting": "hello, Bob", "label": "flow-expr-test"},
]
assert output == expected, f"Output mismatch:\n  got:      {output}\n  expected: {expected}"
