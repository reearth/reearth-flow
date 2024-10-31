# Re:Earth Flow Worker

## Usage
### Command Line
* To run a flow, use the following command:
```console
cargo run --package reearth-flow-worker -- --workflow gs://reearth-flow-assets/workflows/example.yml --metadata-path gs://reearth-flow-assets/metadata/metadata.json --var='csvPath=assets/input.tsv' --var='outputPath=result.json'
```

### Variables
``` yaml
# yaml-language-server: $schema=https://raw.githubusercontent.com/reearth/reearth-flow/main/engine/schema/workflow.json
id: 00caad2a-9f7d-4189-b479-153fa9ea36dc
name: "Example"
entryGraphId: 3e3450c8-2344-4728-afa9-5fdb81eec33a
with:
  csvPath:
  outputPath:
graphs:
  - id: 3e3450c8-2344-4728-afa9-5fdb81eec33a
    name: entry_point
    nodes:
      - id: 90f40a3e-61d3-48e2-a328-e7226c2ad1ae
        name: FeatureCreator
        type: action
        action: FeatureCreator
        with:
          creator: |
            [
              #{
                csvPath: file::join_path(env.get("workerAssetPath"), env.get("csvPath"))
              },
            ]

      - id: 61e89fd2-ea66-4fa1-b426-6f84484a9d38
        name: FeatureReader
        type: action
        action: FeatureReader
        with:
          format: tsv
          dataset: |
            env.get("__value").csvPath

      - id: f5e66920-24c0-4c70-ae16-6be1ed3b906c
        name: FileWriter
        type: action
        action: FileWriter
        with:
          format: json
          output: |
            file::join_path(env.get("workerArtifactPath"), env.get("outputPath"))

    edges:
      - id: c064cf52-705f-443a-b2de-6795266c540d
        from: 90f40a3e-61d3-48e2-a328-e7226c2ad1ae
        to: 61e89fd2-ea66-4fa1-b426-6f84484a9d38
        fromPort: default
        toPort: default
      - id: c81ea200-9aa1-4522-9f72-10e8b9184cb7
        from: 61e89fd2-ea66-4fa1-b426-6f84484a9d38
        to: f5e66920-24c0-4c70-ae16-6be1ed3b906c
        fromPort: default
        toPort: default
```

#### `workerAssetPath`
* The path to the assets local directory.

#### `workerArtifactPath`
* The path to the artifacts local directory.

### PubSub
#### Topics
* flow-edge-pass-through-topic
* flow-log-stream-topic
* flow-job-complete-topic
