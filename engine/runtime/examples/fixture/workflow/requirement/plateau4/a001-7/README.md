## Usage
### Command Line
* To run a flow, use the following command:

#### buffer
``` sh
## Specify the workflow yaml you want to run
export FLOW_EXAMPLE_TARGET_WORKFLOW=fixture/workflow/requirement/plateau4/a001-7/buffer.yml
cargo run --package reearth-flow-examples --example example_main
```

#### convex_hull
``` sh
## Specify the workflow yaml you want to run
export FLOW_EXAMPLE_TARGET_WORKFLOW=fixture/workflow/requirement/plateau4/a001-7/convex_hull_accumulator.yml
cargo run --package reearth-flow-examples --example example_main
```

#### disolver
``` sh
## Specify the workflow yaml you want to run
export FLOW_EXAMPLE_TARGET_WORKFLOW=fixture/workflow/requirement/plateau4/a001-7/dissolver.yml
cargo run --package reearth-flow-examples --example example_main
```
