# Actions

## AreaOnAreaOverlayer
### Type
* processor
### Description
Overlays an area on another area
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "AreaOnAreaOverlayerParam",
  "type": "object",
  "properties": {
    "groupBy": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/$defs/Attribute"
      }
    },
    "outputAttribute": {
      "$ref": "#/$defs/Attribute"
    }
  },
  "required": [
    "outputAttribute"
  ],
  "$defs": {
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* area
* remnants
* rejected
### Category
* Geometry

## AttributeAggregator
### Type
* processor
### Description
Aggregates features by attributes
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "AttributeAggregatorParam",
  "type": "object",
  "properties": {
    "aggregateAttributes": {
      "type": "array",
      "items": {
        "$ref": "#/$defs/AggregateAttribute"
      }
    },
    "calculation": {
      "anyOf": [
        {
          "$ref": "#/$defs/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "calculationValue": {
      "type": [
        "integer",
        "null"
      ],
      "format": "int64"
    },
    "calculationAttribute": {
      "$ref": "#/$defs/Attribute"
    },
    "method": {
      "$ref": "#/$defs/Method"
    }
  },
  "required": [
    "aggregateAttributes",
    "calculationAttribute",
    "method"
  ],
  "$defs": {
    "AggregateAttribute": {
      "type": "object",
      "properties": {
        "newAttribute": {
          "$ref": "#/$defs/Attribute"
        },
        "attribute": {
          "type": [
            "string",
            "null"
          ]
        },
        "attributeValue": {
          "anyOf": [
            {
              "$ref": "#/$defs/Expr"
            },
            {
              "type": "null"
            }
          ]
        }
      },
      "required": [
        "newAttribute"
      ]
    },
    "Attribute": {
      "type": "string"
    },
    "Expr": {
      "type": "string"
    },
    "Method": {
      "type": "string",
      "enum": [
        "max",
        "min",
        "count"
      ]
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Attribute

## AttributeDuplicateFilter
### Type
* processor
### Description
Filters features by duplicate attributes
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "AttributeDuplicateFilterParam",
  "type": "object",
  "properties": {
    "filterBy": {
      "type": "array",
      "items": {
        "$ref": "#/$defs/Attribute"
      }
    }
  },
  "required": [
    "filterBy"
  ],
  "$defs": {
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Attribute

## AttributeFilePathInfoExtractor
### Type
* processor
### Description
Extracts file path information from attributes
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "AttributeFilePathInfoExtractor",
  "type": "object",
  "properties": {
    "attribute": {
      "$ref": "#/$defs/Attribute"
    }
  },
  "required": [
    "attribute"
  ],
  "$defs": {
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
* rejected
### Category
* Attribute

## AttributeManager
### Type
* processor
### Description
Manages attributes
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "AttributeManagerParam",
  "type": "object",
  "properties": {
    "operations": {
      "type": "array",
      "items": {
        "$ref": "#/$defs/Operation"
      }
    }
  },
  "required": [
    "operations"
  ],
  "$defs": {
    "Operation": {
      "type": "object",
      "properties": {
        "attribute": {
          "type": "string"
        },
        "method": {
          "$ref": "#/$defs/Method"
        },
        "value": {
          "anyOf": [
            {
              "$ref": "#/$defs/Expr"
            },
            {
              "type": "null"
            }
          ]
        }
      },
      "required": [
        "attribute",
        "method"
      ]
    },
    "Method": {
      "type": "string",
      "enum": [
        "convert",
        "create",
        "rename",
        "remove"
      ]
    },
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Attribute

## AttributeMapper
### Type
* processor
### Description
Maps attributes
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "AttributeMapperParam",
  "type": "object",
  "properties": {
    "mappers": {
      "type": "array",
      "items": {
        "$ref": "#/$defs/Mapper"
      }
    }
  },
  "required": [
    "mappers"
  ],
  "$defs": {
    "Mapper": {
      "type": "object",
      "properties": {
        "attribute": {
          "type": [
            "string",
            "null"
          ]
        },
        "expr": {
          "anyOf": [
            {
              "$ref": "#/$defs/Expr"
            },
            {
              "type": "null"
            }
          ]
        },
        "valueAttribute": {
          "type": [
            "string",
            "null"
          ]
        },
        "parentAttribute": {
          "type": [
            "string",
            "null"
          ]
        },
        "childAttribute": {
          "type": [
            "string",
            "null"
          ]
        },
        "multipleExpr": {
          "anyOf": [
            {
              "$ref": "#/$defs/Expr"
            },
            {
              "type": "null"
            }
          ]
        }
      }
    },
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Attribute

## BoundsExtractor
### Type
* processor
### Description
Bounds Extractor
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
* rejected
### Category
* Geometry

## Bufferer
### Type
* processor
### Description
Buffers a geometry
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Bufferer",
  "type": "object",
  "properties": {
    "bufferType": {
      "$ref": "#/$defs/BufferType"
    },
    "distance": {
      "type": "number",
      "format": "double"
    },
    "interpolationAngle": {
      "type": "number",
      "format": "double"
    }
  },
  "required": [
    "bufferType",
    "distance",
    "interpolationAngle"
  ],
  "$defs": {
    "BufferType": {
      "type": "string",
      "enum": [
        "area2d"
      ]
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
* rejected
### Category
* Geometry

## BulkAttributeRenamer
### Type
* processor
### Description
Renames attributes by adding/removing prefixes or suffixes, or replacing text
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "BulkAttributeRenamerParam",
  "type": "object",
  "properties": {
    "renameType": {
      "$ref": "#/$defs/RenameType"
    },
    "renameAction": {
      "$ref": "#/$defs/RenameAction"
    },
    "textToFind": {
      "type": [
        "string",
        "null"
      ]
    },
    "renameValue": {
      "type": "string"
    },
    "selectedAttributes": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "type": "string"
      }
    }
  },
  "required": [
    "renameType",
    "renameAction",
    "renameValue"
  ],
  "$defs": {
    "RenameType": {
      "type": "string",
      "enum": [
        "All",
        "Selected"
      ]
    },
    "RenameAction": {
      "type": "string",
      "enum": [
        "AddPrefix",
        "AddSuffix",
        "RemovePrefix",
        "RemoveSuffix",
        "StringReplace"
      ]
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Attribute

## CenterPointReplacer
### Type
* processor
### Description
Replaces the geometry of the feature with a point that is either in the center of the feature's bounding box, at the center of mass of the feature, or somewhere guaranteed to be inside the feature's area.
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* point
* rejected
### Category
* Geometry

## Cesium3DTilesWriter
### Type
* sink
### Description
Writes features to a file
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Cesium3DTilesWriterParam",
  "type": "object",
  "properties": {
    "output": {
      "$ref": "#/$defs/Expr"
    },
    "minZoom": {
      "type": "integer",
      "format": "uint8",
      "minimum": 0
    },
    "maxZoom": {
      "type": "integer",
      "format": "uint8",
      "minimum": 0
    },
    "attachTexture": {
      "type": [
        "boolean",
        "null"
      ]
    }
  },
  "required": [
    "output",
    "minZoom",
    "maxZoom"
  ],
  "$defs": {
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
### Category
* File

## Clipper
### Type
* processor
### Description
Divides Candidate features using Clipper features, so that Candidates and parts of Candidates that are inside or outside of the Clipper features are output separately
### Parameters
* No parameters
### Input Ports
* clipper
* candidate
### Output Ports
* inside
* outside
* rejected
### Category
* Geometry

## ClosedCurveFilter
### Type
* processor
### Description
Checks if curves form closed loops
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* closed
* open
* rejected
### Category
* Geometry

## CoordinateSystemSetter
### Type
* processor
### Description
Sets the coordinate system of a feature
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "CoordinateSystemSetter",
  "type": "object",
  "properties": {
    "epsgCode": {
      "type": "integer",
      "format": "uint16",
      "minimum": 0
    }
  },
  "required": [
    "epsgCode"
  ]
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Geometry

## DimensionFilter
### Type
* processor
### Description
Filters the dimension of features
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* 2d
* 3d
* rejected
### Category
* Geometry

## EchoProcessor
### Type
* processor
### Description
Echo features
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
### Category
* Debug

## EchoSink
### Type
* sink
### Description
Echo features
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
### Category
* Debug

## ElevationExtractor
### Type
* processor
### Description
Extracts a feature’s first z coordinate value, storing it in an attribute.
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "ElevationExtractorParam",
  "type": "object",
  "properties": {
    "outputAttribute": {
      "$ref": "#/$defs/Attribute"
    }
  },
  "required": [
    "outputAttribute"
  ],
  "$defs": {
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Geometry

## Extruder
### Type
* processor
### Description
Extrudes a polygon by a distance
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "ExtruderParam",
  "type": "object",
  "properties": {
    "distance": {
      "$ref": "#/$defs/Expr"
    }
  },
  "required": [
    "distance"
  ],
  "$defs": {
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Geometry

## FeatureCounter
### Type
* processor
### Description
Counts features
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "FeatureCounterParam",
  "type": "object",
  "properties": {
    "countStart": {
      "type": "integer",
      "format": "int64"
    },
    "groupBy": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/$defs/Attribute"
      }
    },
    "outputAttribute": {
      "type": "string"
    }
  },
  "required": [
    "countStart",
    "outputAttribute"
  ],
  "$defs": {
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
* rejected
### Category
* Feature

## FeatureCreator
### Type
* source
### Description
Creates features from expressions
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "FeatureCreator",
  "type": "object",
  "properties": {
    "creator": {
      "$ref": "#/$defs/Expr"
    }
  },
  "required": [
    "creator"
  ],
  "$defs": {
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
### Output Ports
* default
### Category
* Feature

## FeatureFilePathExtractor
### Type
* processor
### Description
Extracts features by file path
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "FeatureFilePathExtractorParam",
  "type": "object",
  "properties": {
    "sourceDataset": {
      "$ref": "#/$defs/Expr"
    },
    "extractArchive": {
      "type": "boolean"
    }
  },
  "required": [
    "sourceDataset",
    "extractArchive"
  ],
  "$defs": {
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
* unfiltered
### Category
* Feature

## FeatureFilter
### Type
* processor
### Description
Filters features based on conditions
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "FeatureFilterParam",
  "type": "object",
  "properties": {
    "conditions": {
      "type": "array",
      "items": {
        "$ref": "#/$defs/Condition"
      }
    }
  },
  "required": [
    "conditions"
  ],
  "$defs": {
    "Condition": {
      "type": "object",
      "properties": {
        "expr": {
          "$ref": "#/$defs/Expr"
        },
        "outputPort": {
          "$ref": "#/$defs/Port"
        }
      },
      "required": [
        "expr",
        "outputPort"
      ]
    },
    "Expr": {
      "type": "string"
    },
    "Port": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* unfiltered
### Category
* Feature

## FeatureMerger
### Type
* processor
### Description
Merges features by attributes
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "FeatureMergerParam",
  "type": "object",
  "properties": {
    "requestorAttribute": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/$defs/Attribute"
      }
    },
    "supplierAttribute": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/$defs/Attribute"
      }
    },
    "requestorAttributeValue": {
      "anyOf": [
        {
          "$ref": "#/$defs/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "supplierAttributeValue": {
      "anyOf": [
        {
          "$ref": "#/$defs/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "completeGrouped": {
      "type": [
        "boolean",
        "null"
      ]
    }
  },
  "$defs": {
    "Attribute": {
      "type": "string"
    },
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
* requestor
* supplier
### Output Ports
* merged
* unmerged
### Category
* Feature

## FeatureReader
### Type
* processor
### Description
Filters features based on conditions
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "FeatureReaderParam",
  "oneOf": [
    {
      "type": "object",
      "properties": {
        "format": {
          "type": "string",
          "const": "citygml"
        },
        "dataset": {
          "$ref": "#/$defs/Expr"
        },
        "flatten": {
          "type": [
            "boolean",
            "null"
          ]
        }
      },
      "required": [
        "format",
        "dataset"
      ]
    },
    {
      "type": "object",
      "properties": {
        "format": {
          "type": "string",
          "const": "csv"
        },
        "dataset": {
          "$ref": "#/$defs/Expr"
        },
        "offset": {
          "type": [
            "integer",
            "null"
          ],
          "format": "uint",
          "minimum": 0
        }
      },
      "required": [
        "format",
        "dataset"
      ]
    },
    {
      "type": "object",
      "properties": {
        "format": {
          "type": "string",
          "const": "tsv"
        },
        "dataset": {
          "$ref": "#/$defs/Expr"
        },
        "offset": {
          "type": [
            "integer",
            "null"
          ],
          "format": "uint",
          "minimum": 0
        }
      },
      "required": [
        "format",
        "dataset"
      ]
    }
  ],
  "$defs": {
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Feature

## FeatureSorter
### Type
* processor
### Description
Sorts features by attributes
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "FeatureSorterParam",
  "type": "object",
  "properties": {
    "sortBy": {
      "type": "array",
      "items": {
        "$ref": "#/$defs/SortBy"
      }
    }
  },
  "required": [
    "sortBy"
  ],
  "$defs": {
    "SortBy": {
      "type": "object",
      "properties": {
        "attribute": {
          "anyOf": [
            {
              "$ref": "#/$defs/Attribute"
            },
            {
              "type": "null"
            }
          ]
        },
        "attributeValue": {
          "anyOf": [
            {
              "$ref": "#/$defs/Expr"
            },
            {
              "type": "null"
            }
          ]
        },
        "order": {
          "$ref": "#/$defs/Order"
        }
      },
      "required": [
        "order"
      ]
    },
    "Attribute": {
      "type": "string"
    },
    "Expr": {
      "type": "string"
    },
    "Order": {
      "type": "string",
      "enum": [
        "ascending",
        "descending"
      ]
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Feature

## FeatureTransformer
### Type
* processor
### Description
Transforms features by expressions
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "FeatureTransformerParam",
  "type": "object",
  "properties": {
    "transformers": {
      "type": "array",
      "items": {
        "$ref": "#/$defs/Transform"
      }
    }
  },
  "required": [
    "transformers"
  ],
  "$defs": {
    "Transform": {
      "type": "object",
      "properties": {
        "expr": {
          "$ref": "#/$defs/Expr"
        }
      },
      "required": [
        "expr"
      ]
    },
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Feature

## FeatureTypeFilter
### Type
* processor
### Description
Filters features by feature type
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "FeatureTypeFilter",
  "type": "object",
  "properties": {
    "targetTypes": {
      "type": "array",
      "items": {
        "type": "string"
      }
    }
  },
  "required": [
    "targetTypes"
  ]
}
```
### Input Ports
* default
### Output Ports
* default
* unfiltered
### Category
* Feature

## FilePathExtractor
### Type
* source
### Description
Extracts files from a directory or an archive
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "FilePathExtractor",
  "type": "object",
  "properties": {
    "sourceDataset": {
      "$ref": "#/$defs/Expr"
    },
    "extractArchive": {
      "type": "boolean"
    }
  },
  "required": [
    "sourceDataset",
    "extractArchive"
  ],
  "$defs": {
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
### Output Ports
* default
### Category
* File

## FilePropertyExtractor
### Type
* processor
### Description
Extracts properties from a file
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "FilePropertyExtractor",
  "type": "object",
  "properties": {
    "filePathAttribute": {
      "type": "string"
    }
  },
  "required": [
    "filePathAttribute"
  ]
}
```
### Input Ports
* default
### Output Ports
* default
* rejected
### Category
* File

## FileReader
### Type
* source
### Description
Reads features from a file
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "FileReader",
  "oneOf": [
    {
      "title": "CSV",
      "type": "object",
      "properties": {
        "format": {
          "type": "string",
          "const": "csv"
        },
        "dataset": {
          "$ref": "#/$defs/Expr"
        },
        "offset": {
          "type": [
            "integer",
            "null"
          ],
          "format": "uint",
          "minimum": 0
        }
      },
      "required": [
        "format",
        "dataset"
      ]
    },
    {
      "title": "TSV",
      "type": "object",
      "properties": {
        "format": {
          "type": "string",
          "const": "tsv"
        },
        "dataset": {
          "$ref": "#/$defs/Expr"
        },
        "offset": {
          "type": [
            "integer",
            "null"
          ],
          "format": "uint",
          "minimum": 0
        }
      },
      "required": [
        "format",
        "dataset"
      ]
    },
    {
      "title": "JSON",
      "type": "object",
      "properties": {
        "format": {
          "type": "string",
          "const": "json"
        },
        "dataset": {
          "$ref": "#/$defs/Expr"
        }
      },
      "required": [
        "format",
        "dataset"
      ]
    },
    {
      "title": "CityGML",
      "type": "object",
      "properties": {
        "format": {
          "type": "string",
          "const": "citygml"
        },
        "dataset": {
          "$ref": "#/$defs/Expr"
        },
        "flatten": {
          "type": [
            "boolean",
            "null"
          ]
        }
      },
      "required": [
        "format",
        "dataset"
      ]
    }
  ],
  "$defs": {
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
### Output Ports
* default
### Category
* File

## FileWriter
### Type
* sink
### Description
Writes features to a file
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "FileWriterParam",
  "oneOf": [
    {
      "type": "object",
      "properties": {
        "format": {
          "type": "string",
          "const": "csv"
        },
        "output": {
          "$ref": "#/$defs/Expr"
        }
      },
      "required": [
        "format",
        "output"
      ]
    },
    {
      "type": "object",
      "properties": {
        "format": {
          "type": "string",
          "const": "tsv"
        },
        "output": {
          "$ref": "#/$defs/Expr"
        }
      },
      "required": [
        "format",
        "output"
      ]
    },
    {
      "type": "object",
      "properties": {
        "format": {
          "type": "string",
          "const": "json"
        },
        "output": {
          "$ref": "#/$defs/Expr"
        },
        "converter": {
          "anyOf": [
            {
              "$ref": "#/$defs/Expr"
            },
            {
              "type": "null"
            }
          ]
        }
      },
      "required": [
        "format",
        "output"
      ]
    },
    {
      "type": "object",
      "properties": {
        "format": {
          "type": "string",
          "const": "excel"
        },
        "output": {
          "$ref": "#/$defs/Expr"
        },
        "sheetName": {
          "type": [
            "string",
            "null"
          ]
        }
      },
      "required": [
        "format",
        "output"
      ]
    }
  ],
  "$defs": {
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
### Category
* File

## GeoJsonWriter
### Type
* sink
### Description
Writes features to a geojson file
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "GeoJsonWriterParam",
  "type": "object",
  "properties": {
    "output": {
      "$ref": "#/$defs/Expr"
    },
    "groupBy": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/$defs/Attribute"
      }
    }
  },
  "required": [
    "output"
  ],
  "$defs": {
    "Expr": {
      "type": "string"
    },
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
### Category
* File

## GeometryCoercer
### Type
* processor
### Description
Coerces the geometry of a feature to a specific geometry
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "GeometryCoercer",
  "type": "object",
  "properties": {
    "coercerType": {
      "$ref": "#/$defs/CoercerType"
    }
  },
  "required": [
    "coercerType"
  ],
  "$defs": {
    "CoercerType": {
      "type": "string",
      "enum": [
        "lineString"
      ]
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Geometry

## GeometryDissolver
### Type
* processor
### Description
Dissolve geometries
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "GeometryDissolverParam",
  "type": "object",
  "properties": {
    "groupBy": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/$defs/Attribute"
      }
    },
    "completeGrouped": {
      "type": [
        "boolean",
        "null"
      ]
    }
  },
  "$defs": {
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* area
* rejected
### Category
* Geometry

## GeometryExtractor
### Type
* processor
### Description
Extracts geometry from a feature and adds it as an attribute.
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "GeometryExtractor",
  "type": "object",
  "properties": {
    "outputAttribute": {
      "$ref": "#/$defs/Attribute"
    }
  },
  "required": [
    "outputAttribute"
  ],
  "$defs": {
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Geometry

## GeometryFilter
### Type
* processor
### Description
Filter geometry by type
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "GeometryFilterParam",
  "oneOf": [
    {
      "type": "object",
      "properties": {
        "filterType": {
          "type": "string",
          "const": "none"
        }
      },
      "required": [
        "filterType"
      ]
    },
    {
      "type": "object",
      "properties": {
        "filterType": {
          "type": "string",
          "const": "multiple"
        }
      },
      "required": [
        "filterType"
      ]
    },
    {
      "type": "object",
      "properties": {
        "filterType": {
          "type": "string",
          "const": "geometryType"
        }
      },
      "required": [
        "filterType"
      ]
    }
  ]
}
```
### Input Ports
* default
### Output Ports
* unfiltered
* none
* contains
* point
* line
* lineString
* polygon
* multiPoint
* multiLineString
* multiPolygon
* rect
* triangle
* solid
* geometryCollection
* solid
* multiSurface
* compositeSurface
* surface
* triangle
* multiCurve
* curve
* multiPoint
* point
* tin
### Category
* Geometry

## GeometryLodFilter
### Type
* processor
### Description
Filter geometry by lod
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "GeometryLodFilterParam",
  "type": "object",
  "properties": {
    "upToLod": {
      "type": [
        "integer",
        "null"
      ],
      "format": "uint8",
      "minimum": 0
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
* unfiltered
### Category
* Geometry

## GeometryReplacer
### Type
* processor
### Description
Replaces the geometry of a feature with a new geometry.
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "GeometryReplacer",
  "type": "object",
  "properties": {
    "sourceAttribute": {
      "$ref": "#/$defs/Attribute"
    }
  },
  "required": [
    "sourceAttribute"
  ],
  "$defs": {
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Geometry

## GeometrySplitter
### Type
* processor
### Description
Split geometry by type
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
### Category
* Geometry

## GeometryValidator
### Type
* processor
### Description
Validates the geometry of a feature
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "GeometryValidator",
  "type": "object",
  "properties": {
    "validationTypes": {
      "type": "array",
      "items": {
        "$ref": "#/$defs/ValidationType"
      }
    }
  },
  "required": [
    "validationTypes"
  ],
  "$defs": {
    "ValidationType": {
      "type": "string",
      "enum": [
        "duplicatePoints",
        "corruptGeometry",
        "selfIntersection"
      ]
    }
  }
}
```
### Input Ports
* default
### Output Ports
* success
* failed
* rejected
### Category
* Geometry

## GeometryValueFilter
### Type
* processor
### Description
Filter geometry by value
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* none
* geometry2d
* geometry3d
* cityGml
### Category
* Geometry

## GltfWriter
### Type
* sink
### Description
Writes features to a Gltf
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "GltfWriterParam",
  "type": "object",
  "properties": {
    "output": {
      "$ref": "#/$defs/Expr"
    },
    "attachTexture": {
      "type": [
        "boolean",
        "null"
      ]
    }
  },
  "required": [
    "output"
  ],
  "$defs": {
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
### Category
* File

## HoleCounter
### Type
* processor
### Description
Counts the number of holes in a geometry and adds it as an attribute.
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "HoleCounterParam",
  "type": "object",
  "properties": {
    "outputAttribute": {
      "$ref": "#/$defs/Attribute"
    }
  },
  "required": [
    "outputAttribute"
  ],
  "$defs": {
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Geometry

## HoleExtractor
### Type
* processor
### Description
Extracts holes in a geometry and adds it as an attribute.
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* outershell
* hole
* rejected
### Category
* Geometry

## HorizontalReprojector
### Type
* processor
### Description
Reprojects the geometry of a feature to a specified coordinate system
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "HorizontalReprojectorParam",
  "type": "object",
  "properties": {
    "epsgCode": {
      "type": [
        "integer",
        "null"
      ],
      "format": "uint16",
      "minimum": 0
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Geometry

## InputRouter
### Type
* processor
### Description
Action for first port forwarding for sub-workflows.
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "InputRouter",
  "type": "object",
  "properties": {
    "routingPort": {
      "type": "string"
    }
  },
  "required": [
    "routingPort"
  ]
}
```
### Input Ports
### Output Ports
* default
### Category
* System

## LineOnLineOverlayer
### Type
* processor
### Description
Intersection points are turned into point features that can contain the merged list of attributes of the original intersected lines.
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "LineOnLineOverlayerParam",
  "type": "object",
  "properties": {
    "groupBy": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/$defs/Attribute"
      }
    },
    "outputAttribute": {
      "$ref": "#/$defs/Attribute"
    }
  },
  "required": [
    "outputAttribute"
  ],
  "$defs": {
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* point
* line
* rejected
### Category
* Geometry

## ListExploder
### Type
* processor
### Description
Explodes list attributes
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "ListExploder",
  "type": "object",
  "properties": {
    "sourceAttribute": {
      "$ref": "#/$defs/Attribute"
    }
  },
  "required": [
    "sourceAttribute"
  ],
  "$defs": {
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Feature

## MVTWriter
### Type
* sink
### Description
Writes features to a file
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "MVTWriterParam",
  "type": "object",
  "properties": {
    "output": {
      "$ref": "#/$defs/Expr"
    },
    "layerName": {
      "$ref": "#/$defs/Expr"
    },
    "minZoom": {
      "type": "integer",
      "format": "uint8",
      "minimum": 0
    },
    "maxZoom": {
      "type": "integer",
      "format": "uint8",
      "minimum": 0
    }
  },
  "required": [
    "output",
    "layerName",
    "minZoom",
    "maxZoom"
  ],
  "$defs": {
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
### Category
* File

## NoopProcessor
### Type
* processor
### Description
Noop features
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
### Category
* Noop

## NoopSink
### Type
* sink
### Description
noop sink
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
### Category
* Noop

## Offsetter
### Type
* processor
### Description
Adds offsets to the feature's coordinates.
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "OffsetterParam",
  "type": "object",
  "properties": {
    "offsetX": {
      "type": [
        "number",
        "null"
      ],
      "format": "double"
    },
    "offsetY": {
      "type": [
        "number",
        "null"
      ],
      "format": "double"
    },
    "offsetZ": {
      "type": [
        "number",
        "null"
      ],
      "format": "double"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Geometry

## OrientationExtractor
### Type
* processor
### Description
Extracts the orientation of a geometry from a feature and adds it as an attribute.
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "OrientationExtractorParam",
  "type": "object",
  "properties": {
    "outputAttribute": {
      "$ref": "#/$defs/Attribute"
    }
  },
  "required": [
    "outputAttribute"
  ],
  "$defs": {
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Geometry

## OutputRouter
### Type
* processor
### Description
Action for last port forwarding for sub-workflows.
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "OutputRouter",
  "type": "object",
  "properties": {
    "routingPort": {
      "type": "string"
    }
  },
  "required": [
    "routingPort"
  ]
}
```
### Input Ports
* default
### Output Ports
### Category
* System

## PLATEAU3.AttributeFlattener
### Type
* processor
### Description
Flatten attributes for building feature
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
### Category
* PLATEAU

## PLATEAU3.BuildingInstallationGeometryTypeExtractor
### Type
* processor
### Description
Extracts BuildingInstallationGeometryType
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
### Category
* PLATEAU

## PLATEAU3.BuildingUsageAttributeValidator
### Type
* processor
### Description
This processor validates building usage attributes by checking for the presence of required attributes and ensuring the correctness of city codes. It outputs errors through the lBldgError and codeError ports if any issues are found.
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* lBldgError
* codeError
* default
### Category
* PLATEAU

## PLATEAU3.DictionariesInitiator
### Type
* processor
### Description
Initializes dictionaries for PLATEAU
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
* rejected
### Category
* PLATEAU

## PLATEAU3.DomainOfDefinitionValidator
### Type
* processor
### Description
Validates domain of definition of CityGML features
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
* rejected
### Category
* PLATEAU

## PLATEAU3.MaxLodExtractor
### Type
* processor
### Description
Extracts maxLod
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
### Category
* PLATEAU

## PLATEAU3.TranXLinkChecker
### Type
* processor
### Description
Check Xlink for Tran
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
### Category
* PLATEAU

## PLATEAU3.UDXFolderExtractor
### Type
* processor
### Description
Extracts UDX folders from cityGML path
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
* rejected
### Category
* PLATEAU

## PLATEAU3.UnmatchedXlinkDetector
### Type
* processor
### Description
Detect unmatched xlink for PLATEAU
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* summary
* unMatchedXlinkFrom
* unMatchedXlinkTo
### Category
* PLATEAU

## PLATEAU3.XMLAttributeExtractor
### Type
* processor
### Description
Extracts attributes from XML fragments based on a schema definition
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* attributeFeature
* summary
* filePath
### Category
* PLATEAU

## PLATEAU4.UDXFolderExtractor
### Type
* processor
### Description
Extracts UDX folders from cityGML path
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "UDXFolderExtractorParam",
  "type": "object",
  "properties": {
    "cityGmlPath": {
      "$ref": "#/$defs/Expr"
    }
  },
  "required": [
    "cityGmlPath"
  ],
  "$defs": {
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
* rejected
### Category
* PLATEAU

## PlanarityFilter
### Type
* processor
### Description
Filter geometry by type
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* planarity
* notplanarity
### Category
* Geometry

## Refiner
### Type
* processor
### Description
Geometry Refiner
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
* remain
### Category
* Geometry

## RhaiCaller
### Type
* processor
### Description
Calls Rhai script
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "RhaiCallerParam",
  "type": "object",
  "properties": {
    "isTarget": {
      "$ref": "#/$defs/Expr"
    },
    "process": {
      "$ref": "#/$defs/Expr"
    }
  },
  "required": [
    "isTarget",
    "process"
  ],
  "$defs": {
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Feature

## StatisticsCalculator
### Type
* processor
### Description
Calculates statistics of features
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "StatisticsCalculatorParam",
  "type": "object",
  "properties": {
    "aggregateName": {
      "anyOf": [
        {
          "$ref": "#/$defs/Attribute"
        },
        {
          "type": "null"
        }
      ]
    },
    "aggregateAttribute": {
      "anyOf": [
        {
          "$ref": "#/$defs/Attribute"
        },
        {
          "type": "null"
        }
      ]
    },
    "calculations": {
      "type": "array",
      "items": {
        "$ref": "#/$defs/Calculation"
      }
    }
  },
  "required": [
    "calculations"
  ],
  "$defs": {
    "Attribute": {
      "type": "string"
    },
    "Calculation": {
      "type": "object",
      "properties": {
        "newAttribute": {
          "$ref": "#/$defs/Attribute"
        },
        "expr": {
          "$ref": "#/$defs/Expr"
        }
      },
      "required": [
        "newAttribute",
        "expr"
      ]
    },
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
* complete
### Category
* Attribute

## ThreeDimensionBoxReplacer
### Type
* processor
### Description
Replaces a three Dimension box with a polygon.
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "ThreeDimensionBoxReplacer",
  "type": "object",
  "properties": {
    "minX": {
      "$ref": "#/$defs/Attribute"
    },
    "minY": {
      "$ref": "#/$defs/Attribute"
    },
    "minZ": {
      "$ref": "#/$defs/Attribute"
    },
    "maxX": {
      "$ref": "#/$defs/Attribute"
    },
    "maxY": {
      "$ref": "#/$defs/Attribute"
    },
    "maxZ": {
      "$ref": "#/$defs/Attribute"
    }
  },
  "required": [
    "minX",
    "minY",
    "minZ",
    "maxX",
    "maxY",
    "maxZ"
  ],
  "$defs": {
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Geometry

## ThreeDimensionRotator
### Type
* processor
### Description
Replaces a three Dimension box with a polygon.
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "ThreeDimensionRotatorParam",
  "type": "object",
  "properties": {
    "angleDegree": {
      "$ref": "#/$defs/Expr"
    },
    "originX": {
      "$ref": "#/$defs/Expr"
    },
    "originY": {
      "$ref": "#/$defs/Expr"
    },
    "originZ": {
      "$ref": "#/$defs/Expr"
    },
    "directionX": {
      "$ref": "#/$defs/Expr"
    },
    "directionY": {
      "$ref": "#/$defs/Expr"
    },
    "directionZ": {
      "$ref": "#/$defs/Expr"
    }
  },
  "required": [
    "angleDegree",
    "originX",
    "originY",
    "originZ",
    "directionX",
    "directionY",
    "directionZ"
  ],
  "$defs": {
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Geometry

## TwoDimensionForcer
### Type
* processor
### Description
Forces a geometry to be two dimensional.
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
### Category
* Geometry

## VertexRemover
### Type
* processor
### Description
Removes specific vertices from a feature’s geometry
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
* rejected
### Category
* Geometry

## VerticalReprojector
### Type
* processor
### Description
Reprojects the geometry of a feature to a specified coordinate system
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "VerticalReprojectorParam",
  "type": "object",
  "properties": {
    "reprojectorType": {
      "$ref": "#/$defs/VerticalReprojectorType"
    }
  },
  "required": [
    "reprojectorType"
  ],
  "$defs": {
    "VerticalReprojectorType": {
      "type": "string",
      "enum": [
        "jgd2011ToWgs84"
      ]
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Geometry

## WasmRuntimeExecutor
### Type
* processor
### Description
Compiles scripts into .wasm and runs at the wasm runtime
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "WasmRuntimeExecutorParam",
  "type": "object",
  "properties": {
    "sourceCodeFilePath": {
      "type": "string"
    },
    "processorType": {
      "$ref": "#/$defs/ProcessorType"
    },
    "programmingLanguage": {
      "$ref": "#/$defs/ProgrammingLanguage"
    }
  },
  "required": [
    "sourceCodeFilePath",
    "processorType",
    "programmingLanguage"
  ],
  "$defs": {
    "ProcessorType": {
      "type": "string",
      "enum": [
        "Attribute"
      ]
    },
    "ProgrammingLanguage": {
      "type": "string",
      "enum": [
        "Python"
      ]
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* Wasm

## XMLFragmenter
### Type
* processor
### Description
Fragment XML
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "XmlFragmenterParam",
  "oneOf": [
    {
      "type": "object",
      "properties": {
        "elementsToMatch": {
          "$ref": "#/$defs/Expr"
        },
        "elementsToExclude": {
          "$ref": "#/$defs/Expr"
        },
        "attribute": {
          "$ref": "#/$defs/Attribute"
        },
        "source": {
          "type": "string",
          "const": "url"
        }
      },
      "required": [
        "source",
        "elementsToMatch",
        "elementsToExclude",
        "attribute"
      ]
    }
  ],
  "$defs": {
    "Expr": {
      "type": "string"
    },
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* default
### Category
* XML

## XMLValidator
### Type
* processor
### Description
Validates XML content
### Parameters
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "XmlValidatorParam",
  "type": "object",
  "properties": {
    "attribute": {
      "$ref": "#/$defs/Attribute"
    },
    "inputType": {
      "$ref": "#/$defs/XmlInputType"
    },
    "validationType": {
      "$ref": "#/$defs/ValidationType"
    }
  },
  "required": [
    "attribute",
    "inputType",
    "validationType"
  ],
  "$defs": {
    "Attribute": {
      "type": "string"
    },
    "XmlInputType": {
      "type": "string",
      "enum": [
        "file",
        "text"
      ]
    },
    "ValidationType": {
      "type": "string",
      "enum": [
        "syntax",
        "syntaxAndNamespace",
        "syntaxAndSchema"
      ]
    }
  }
}
```
### Input Ports
* default
### Output Ports
* success
* failed
### Category
* PLATEAU
