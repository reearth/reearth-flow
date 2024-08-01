# Node

``` yaml
- id: b1a91180-ab88-4c1a-aab5-48c242a218cb
  name: FeatureFilter01
  type: action
  action: FeatureFilter
  with:
    conditions:
    - expr: |
        env.get("__value").extension == "gml"
    outputPort: default

- id: d3773442-1ba8-47c1-b7c1-0bafa23adec9
  name: AttributeReader01
  type: subGraph
  subGraphId: 64931277-3a82-4a1c-88bc-9b54fe172518
```

## id
* Type: UUID
* Description: ID of the node.

## name
* Type: String
* Description: Name of the node.

## type
* Type: String
* Description: Type of the node.
* action or subGraph

## action
* Type: String
* Description: Action of the node.
* Specify the action to be executed by the node.
* only when type is action

## subGraphId
* Type: UUID
* Description: ID of the subGraph.
* Specify the ID of the subGraph to be executed by the node.
* only when type is subGraph

## with
* Type: Object
* Description: Parameters for the node.
* Specify the parameters required for the action or subGraph.
