# Graph

A graph is a data structure that consists of a finite set of nodes (or vertices) and a set of edges connecting these nodes. A graph can be directed or undirected, and it can be weighted or unweighted.

``` yaml
- id: c6863b71-953b-4d15-af56-396fc93fc617
  name: folder_and_file_path_reader
  nodes:
  - id: b1a91180-ab88-4c1a-aab5-48c242a218ca
    name: FilePathExtractor01
    type: action
    action: FilePathExtractor
    with:
      sourceDataset: |
        env.get("cityGmlPath")
      extractArchive: true
  - id: b1a91180-ab88-4c1a-aab5-48c242a218cb
    name: FeatureFilter01
    type: action
    action: FeatureFilter
    with:
      conditions:
      - expr: |
          env.get("__value").extension == "gml"
        outputPort: default
```

## id
* Type: UUID
* Description: ID of the graph.

## name
* Type: String
* Description: Name of the graph.

## nodes
* Type: Array
* Description: Nodes in the graph.


