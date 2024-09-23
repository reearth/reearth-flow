# workflow

``` yaml
id: fefb4c7e-3b0e-4a34-a672-cb564cefa14a
name: QualityCheck-02-t-bldg-06
entryGraphId: 3e3450c8-2344-4728-afa9-5fdb81eec33a
with:
  cityGmlPath: https://assets.cms.plateau.reearth.io/assets/ab/73d159-7d5a-4fd7-a48a-cc585157d03f/11234_yashio-shi_city_2023_citygml_1_op.zip
  cityCode: "11234"
  codelistsPath: null
  schemasPath: null
  targetPackages:
  - bldg
  addNsprefixToFeatureTypes: true
  extractDmGeometryAsXmlFragment: false
graphs:
```

## id
* Type: UUID
* Description: ID of the workflow.

## name
* Type: String
* Description: Name of the workflow.

## entryGraphId
* Type: UUID
* Description: ID of the entry graph.
* Specify the ID of the graph that is the entry point in the graphs.

## with
* Type: Object
* Description: Parameters for the workflow.

<div class="warning">
If the parameter name is not declared with or below, the workflow parameter will never be set, even if the parameter is specified by a command line argument or environment variable.
</div>

### Worfkflow variables on the Command Line
* To specify individual variables on the command line, use the -var option when running the

``` console
$ cargo run -- run --var="cityGmlPath=file:///root/53395658_bldg_6697_op.gml"
$ cargo run -- run --var='cityGmlPath_targetPackages=["bldg","tran"]'"
```

### Workflow variables on Environment Variables
* As a fallback for the other ways of defining variables, Flow searches the environment of its own process for environment variables named FLOW_VAR_ followed by the name of a declared variable.

```console
export FLOW_VAR_cityGmlPath="file:///root/53395658_bldg_6697_op.gml"
export FLOW_VAR_targetPackages='["bldg", "fld"]'
```

