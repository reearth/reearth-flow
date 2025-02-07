#!/bin/bash
outputPath=$1
if [ -z "$outputPath" ]; then
    echo "Usage: $0 <outputPath>"
    exit 1
fi

yaml-include runtime/tests/fixture/workflow/geometry/dissolver.yml | \
cargo run --package reearth-flow-cli -- run --var="outputPath=${outputPath}" --workflow -
