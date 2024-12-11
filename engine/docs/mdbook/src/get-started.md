# Get Started

## Install toolchains
- Rust (stable)

### Install prerequisites

### Linux/Debian

On linux systems you'd need the development headers of libxml2 (e.g. `libxml2-dev` in Debian), as well as `pkg-config`.

### MacOS
```
$ brew install libxml2 pkg-config
$ echo $PKG_CONFIG_PATH
```

### Windows
```
C:\> git clone https://github.com/microsoft/vcpkg
C:\> .\vcpkg\bootstrap-vcpkg.bat
C:\> setx /M PATH "%PATH%;c:\vcpkg" && setx VCPKGRS_DYNAMIC "1" /M
C:\> refreshenv
C:\> vcpkg install libxml2:x64-windows
C:\> vcpkg integrate install
```

## Create workflow
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
  - id: b1a91180-ab88-4c1a-aab5-48c242a218cc
    name: PLATEAU.UDXFolderExtractor-01
    type: action
    action: PLATEAU.UDXFolderExtractor
    with:
      cityGmlPath: |
        env.get("__value")["path"]
  - id: b1a91180-ab88-4c1a-aab5-48c242a218cd
    name: FeatureFilter02
    type: action
    action: FeatureFilter
    with:
      conditions:
      - expr: |
          (env.get("targetPackages") ?? []).is_empty() || env.get("__value")["package"] in env.get("targetPackages")
        outputPort: default
  - id: b1a91180-ab88-4c1a-aab5-48c242a218ce
    name: FeatureCounter01
    type: action
    action: FeatureCounter
    with:
      countStart: 1
      groupBy:
      - udxDirs
      outputAttribute: fileIndex
  - id: cd896e30-7f0e-4c12-90ed-9471ff6049bf
    name: Router
    type: action
    action: OutputRouter
    with:
      routingPort: default
  edges:
  - id: 1379a497-9e4e-40fb-8361-d2eeeb491762
    from: b1a91180-ab88-4c1a-aab5-48c242a218ca
    to: b1a91180-ab88-4c1a-aab5-48c242a218cb
    fromPort: default
    toPort: default
  - id: 2379a497-9e4e-40fb-8361-d2eeeb491763
    from: b1a91180-ab88-4c1a-aab5-48c242a218cb
    to: b1a91180-ab88-4c1a-aab5-48c242a218cc
    fromPort: default
    toPort: default
  - id: 2379a497-9e4e-40fb-8361-d2eeeb491764
    from: b1a91180-ab88-4c1a-aab5-48c242a218cc
    to: b1a91180-ab88-4c1a-aab5-48c242a218cd
    fromPort: default
    toPort: default
  - id: 2379a497-9e4e-40fb-8361-d2eeeb491766
    from: b1a91180-ab88-4c1a-aab5-48c242a218cd
    to: b1a91180-ab88-4c1a-aab5-48c242a218ce
    fromPort: default
    toPort: default
  - id: 80462b53-a06a-4e0b-bed8-07dcda744a55
    from: b1a91180-ab88-4c1a-aab5-48c242a218ce
    to: cd896e30-7f0e-4c12-90ed-9471ff6049bf
    fromPort: default
    toPort: default
- id: 3e3450c8-2344-4728-afa9-5fdb81eec33a
  name: entry_point
  nodes:
  - id: d3773442-1ba8-47c1-b7c1-0bafa23adec9
    name: FolderAndfilePathReader01
    type: subGraph
    subGraphId: c6863b71-953b-4d15-af56-396fc93fc617
  - id: 278ab965-ce22-473d-98c6-c7b381c38679
    name: unmatchedXlinkDetector
    type: action
    action: PLATEAU.UnmatchedXlinkDetector
    with:
      attribute: cityGmlPath
  - id: f5e66920-24c0-4c70-ae16-6be1ed3b906c
    name: echo01
    type: action
    action: EchoSink
  edges:
  - id: c064cf52-705f-443a-b2de-6795266c540d
    from: d3773442-1ba8-47c1-b7c1-0bafa23adec9
    to: 278ab965-ce22-473d-98c6-c7b381c38679
    fromPort: default
    toPort: default
  - id: f23b1f56-c5d8-4311-9239-6dd205b538ab
    from: 278ab965-ce22-473d-98c6-c7b381c38679
    to: f5e66920-24c0-4c70-ae16-6be1ed3b906c
    fromPort: summary
    toPort: default
```

## Run workflow
``` shell
$ cargo run -- run --workflow <path-to-workflow-file>
```
