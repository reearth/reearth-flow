# Actions

## AreaOnAreaOverlayer
### Type
* processor
### Description
Overlays an area on another area
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AreaOnAreaOverlayerParam",
  "type": "object",
  "properties": {
    "groupBy": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AttributeAggregatorParam",
  "type": "object",
  "required": [
    "aggregateAttributes",
    "calculationAttribute",
    "method"
  ],
  "properties": {
    "aggregateAttributes": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/AggregateAttribute"
      }
    },
    "calculation": {
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "calculationAttribute": {
      "$ref": "#/definitions/Attribute"
    },
    "calculationValue": {
      "type": [
        "integer",
        "null"
      ],
      "format": "int64"
    },
    "method": {
      "$ref": "#/definitions/Method"
    }
  },
  "definitions": {
    "AggregateAttribute": {
      "type": "object",
      "required": [
        "newAttribute"
      ],
      "properties": {
        "attribute": {
          "anyOf": [
            {
              "$ref": "#/definitions/Attribute"
            },
            {
              "type": "null"
            }
          ]
        },
        "attributeValue": {
          "anyOf": [
            {
              "$ref": "#/definitions/Expr"
            },
            {
              "type": "null"
            }
          ]
        },
        "newAttribute": {
          "$ref": "#/definitions/Attribute"
        }
      }
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

## AttributeBulkArrayJoiner
### Type
* processor
### Description
Flattens features by attributes
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AttributeBulkArrayJoinerParam",
  "type": "object",
  "properties": {
    "ignoreAttributes": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    }
  },
  "definitions": {
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

## AttributeDuplicateFilter
### Type
* processor
### Description
Filters features by duplicate attributes
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AttributeDuplicateFilterParam",
  "type": "object",
  "required": [
    "filterBy"
  ],
  "properties": {
    "filterBy": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AttributeFilePathInfoExtractor",
  "type": "object",
  "required": [
    "attribute"
  ],
  "properties": {
    "attribute": {
      "$ref": "#/definitions/Attribute"
    }
  },
  "definitions": {
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

## AttributeFlattener
### Type
* processor
### Description
Flattens features by attributes
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AttributeFlattenerParam",
  "type": "object",
  "required": [
    "attributes"
  ],
  "properties": {
    "attributes": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    }
  },
  "definitions": {
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

## AttributeManager
### Type
* processor
### Description
Manages attributes
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AttributeManagerParam",
  "type": "object",
  "required": [
    "operations"
  ],
  "properties": {
    "operations": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/Operation"
      }
    }
  },
  "definitions": {
    "Expr": {
      "type": "string"
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
    "Operation": {
      "type": "object",
      "required": [
        "attribute",
        "method"
      ],
      "properties": {
        "attribute": {
          "type": "string"
        },
        "method": {
          "$ref": "#/definitions/Method"
        },
        "value": {
          "anyOf": [
            {
              "$ref": "#/definitions/Expr"
            },
            {
              "type": "null"
            }
          ]
        }
      }
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AttributeMapperParam",
  "type": "object",
  "required": [
    "mappers"
  ],
  "properties": {
    "mappers": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/Mapper"
      }
    }
  },
  "definitions": {
    "Expr": {
      "type": "string"
    },
    "Mapper": {
      "type": "object",
      "properties": {
        "attribute": {
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
        "expr": {
          "anyOf": [
            {
              "$ref": "#/definitions/Expr"
            },
            {
              "type": "null"
            }
          ]
        },
        "multipleExpr": {
          "anyOf": [
            {
              "$ref": "#/definitions/Expr"
            },
            {
              "type": "null"
            }
          ]
        },
        "parentAttribute": {
          "type": [
            "string",
            "null"
          ]
        },
        "valueAttribute": {
          "type": [
            "string",
            "null"
          ]
        }
      }
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Bufferer",
  "type": "object",
  "required": [
    "bufferType",
    "distance",
    "interpolationAngle"
  ],
  "properties": {
    "bufferType": {
      "$ref": "#/definitions/BufferType"
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
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "BulkAttributeRenamerParam",
  "type": "object",
  "required": [
    "renameAction",
    "renameType",
    "renameValue"
  ],
  "properties": {
    "renameAction": {
      "$ref": "#/definitions/RenameAction"
    },
    "renameType": {
      "$ref": "#/definitions/RenameType"
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
    },
    "textToFind": {
      "type": [
        "string",
        "null"
      ]
    }
  },
  "definitions": {
    "RenameAction": {
      "type": "string",
      "enum": [
        "AddPrefix",
        "AddSuffix",
        "RemovePrefix",
        "RemoveSuffix",
        "StringReplace"
      ]
    },
    "RenameType": {
      "type": "string",
      "enum": [
        "All",
        "Selected"
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Cesium3DTilesWriterParam",
  "type": "object",
  "required": [
    "maxZoom",
    "minZoom",
    "output"
  ],
  "properties": {
    "attachTexture": {
      "type": [
        "boolean",
        "null"
      ]
    },
    "compressOutput": {
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "maxZoom": {
      "type": "integer",
      "format": "uint8",
      "minimum": 0.0
    },
    "minZoom": {
      "type": "integer",
      "format": "uint8",
      "minimum": 0.0
    },
    "output": {
      "$ref": "#/definitions/Expr"
    }
  },
  "definitions": {
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

## ConvexHullAccumulator
### Type
* processor
### Description
Creates a convex hull based on a group of input features.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ConvexHullAccumulatorParam",
  "type": "object",
  "properties": {
    "groupBy": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    }
  },
  "definitions": {
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
* Geometry

## CoordinateSystemSetter
### Type
* processor
### Description
Sets the coordinate system of a feature
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CoordinateSystemSetter",
  "type": "object",
  "required": [
    "epsgCode"
  ],
  "properties": {
    "epsgCode": {
      "type": "integer",
      "format": "uint16",
      "minimum": 0.0
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

## CzmlWriter
### Type
* sink
### Description
Writes features to a Czml file
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CzmlWriterParam",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "groupBy": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "output": {
      "$ref": "#/definitions/Expr"
    }
  },
  "definitions": {
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
* default
### Output Ports
### Category
* File

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

## DirectoryDecompressor
### Type
* processor
### Description
Decompresses a directory
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DirectoryDecompressorParam",
  "type": "object",
  "required": [
    "archiveAttributes"
  ],
  "properties": {
    "archiveAttributes": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    }
  },
  "definitions": {
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
* File

## Dissolver
### Type
* processor
### Description
Dissolves features grouped by specified attributes
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DissolverParam",
  "type": "object",
  "properties": {
    "groupBy": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ElevationExtractorParam",
  "type": "object",
  "required": [
    "outputAttribute"
  ],
  "properties": {
    "outputAttribute": {
      "$ref": "#/definitions/Attribute"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ExtruderParam",
  "type": "object",
  "required": [
    "distance"
  ],
  "properties": {
    "distance": {
      "$ref": "#/definitions/Expr"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureCounterParam",
  "type": "object",
  "required": [
    "countStart",
    "outputAttribute"
  ],
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
        "$ref": "#/definitions/Attribute"
      }
    },
    "outputAttribute": {
      "type": "string"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureCreator",
  "type": "object",
  "required": [
    "creator"
  ],
  "properties": {
    "creator": {
      "$ref": "#/definitions/Expr"
    }
  },
  "definitions": {
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

## FeatureDuplicateFilter
### Type
* processor
### Description
Filters features by duplicate feature
### Parameters
* No parameters
### Input Ports
* default
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureFilePathExtractorParam",
  "type": "object",
  "required": [
    "extractArchive",
    "sourceDataset"
  ],
  "properties": {
    "destPrefix": {
      "type": [
        "string",
        "null"
      ]
    },
    "extractArchive": {
      "type": "boolean"
    },
    "sourceDataset": {
      "$ref": "#/definitions/Expr"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureFilterParam",
  "type": "object",
  "required": [
    "conditions"
  ],
  "properties": {
    "conditions": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/Condition"
      }
    }
  },
  "definitions": {
    "Condition": {
      "type": "object",
      "required": [
        "expr",
        "outputPort"
      ],
      "properties": {
        "expr": {
          "$ref": "#/definitions/Expr"
        },
        "outputPort": {
          "$ref": "#/definitions/Port"
        }
      }
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

## FeatureLodFilter
### Type
* processor
### Description
Filter Geometry by lod
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureLodFilterParam",
  "type": "object",
  "required": [
    "filterKey"
  ],
  "properties": {
    "filterKey": {
      "$ref": "#/definitions/Attribute"
    }
  },
  "definitions": {
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* up_to_lod1
* up_to_lod2
* up_to_lod3
* up_to_lod4
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureMergerParam",
  "type": "object",
  "properties": {
    "completeGrouped": {
      "type": [
        "boolean",
        "null"
      ]
    },
    "requestorAttribute": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "requestorAttributeValue": {
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "supplierAttribute": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "supplierAttributeValue": {
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    }
  },
  "definitions": {
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
Reads features from various formats
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureReaderParam",
  "oneOf": [
    {
      "type": "object",
      "required": [
        "dataset",
        "format"
      ],
      "properties": {
        "dataset": {
          "$ref": "#/definitions/Expr"
        },
        "flatten": {
          "type": [
            "boolean",
            "null"
          ]
        },
        "format": {
          "type": "string",
          "enum": [
            "citygml"
          ]
        }
      }
    },
    {
      "type": "object",
      "required": [
        "dataset",
        "format"
      ],
      "properties": {
        "dataset": {
          "$ref": "#/definitions/Expr"
        },
        "format": {
          "type": "string",
          "enum": [
            "csv"
          ]
        },
        "offset": {
          "type": [
            "integer",
            "null"
          ],
          "format": "uint",
          "minimum": 0.0
        }
      }
    },
    {
      "type": "object",
      "required": [
        "dataset",
        "format"
      ],
      "properties": {
        "dataset": {
          "$ref": "#/definitions/Expr"
        },
        "format": {
          "type": "string",
          "enum": [
            "tsv"
          ]
        },
        "offset": {
          "type": [
            "integer",
            "null"
          ],
          "format": "uint",
          "minimum": 0.0
        }
      }
    },
    {
      "type": "object",
      "required": [
        "dataset",
        "format"
      ],
      "properties": {
        "dataset": {
          "$ref": "#/definitions/Expr"
        },
        "format": {
          "type": "string",
          "enum": [
            "json"
          ]
        }
      }
    }
  ],
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureSorterParam",
  "type": "object",
  "required": [
    "sortBy"
  ],
  "properties": {
    "sortBy": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/SortBy"
      }
    }
  },
  "definitions": {
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
    },
    "SortBy": {
      "type": "object",
      "required": [
        "order"
      ],
      "properties": {
        "attribute": {
          "anyOf": [
            {
              "$ref": "#/definitions/Attribute"
            },
            {
              "type": "null"
            }
          ]
        },
        "attributeValue": {
          "anyOf": [
            {
              "$ref": "#/definitions/Expr"
            },
            {
              "type": "null"
            }
          ]
        },
        "order": {
          "$ref": "#/definitions/Order"
        }
      }
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureTransformerParam",
  "type": "object",
  "required": [
    "transformers"
  ],
  "properties": {
    "transformers": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/Transform"
      }
    }
  },
  "definitions": {
    "Expr": {
      "type": "string"
    },
    "Transform": {
      "type": "object",
      "required": [
        "expr"
      ],
      "properties": {
        "expr": {
          "$ref": "#/definitions/Expr"
        }
      }
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureTypeFilter",
  "type": "object",
  "required": [
    "targetTypes"
  ],
  "properties": {
    "targetTypes": {
      "type": "array",
      "items": {
        "type": "string"
      }
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

## FeatureWriter
### Type
* processor
### Description
Writes features from various formats
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureWriterParam",
  "oneOf": [
    {
      "type": "object",
      "required": [
        "format",
        "output"
      ],
      "properties": {
        "format": {
          "type": "string",
          "enum": [
            "csv"
          ]
        },
        "output": {
          "$ref": "#/definitions/Expr"
        }
      }
    },
    {
      "type": "object",
      "required": [
        "format",
        "output"
      ],
      "properties": {
        "format": {
          "type": "string",
          "enum": [
            "tsv"
          ]
        },
        "output": {
          "$ref": "#/definitions/Expr"
        }
      }
    },
    {
      "type": "object",
      "required": [
        "format",
        "output"
      ],
      "properties": {
        "format": {
          "type": "string",
          "enum": [
            "json"
          ]
        },
        "output": {
          "$ref": "#/definitions/Expr"
        }
      }
    }
  ],
  "definitions": {
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

## FilePathExtractor
### Type
* source
### Description
Extracts files from a directory or an archive
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FilePathExtractor",
  "type": "object",
  "required": [
    "extractArchive",
    "sourceDataset"
  ],
  "properties": {
    "extractArchive": {
      "type": "boolean"
    },
    "sourceDataset": {
      "$ref": "#/definitions/Expr"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FilePropertyExtractor",
  "type": "object",
  "required": [
    "filePathAttribute"
  ],
  "properties": {
    "filePathAttribute": {
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
* File

## FileReader
### Type
* source
### Description
Reads features from a file
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FileReader",
  "oneOf": [
    {
      "title": "CSV",
      "type": "object",
      "required": [
        "format"
      ],
      "properties": {
        "dataset": {
          "anyOf": [
            {
              "$ref": "#/definitions/Expr"
            },
            {
              "type": "null"
            }
          ]
        },
        "format": {
          "type": "string",
          "enum": [
            "csv"
          ]
        },
        "inline": {
          "anyOf": [
            {
              "$ref": "#/definitions/Expr"
            },
            {
              "type": "null"
            }
          ]
        },
        "offset": {
          "type": [
            "integer",
            "null"
          ],
          "format": "uint",
          "minimum": 0.0
        }
      }
    },
    {
      "title": "TSV",
      "type": "object",
      "required": [
        "format"
      ],
      "properties": {
        "dataset": {
          "anyOf": [
            {
              "$ref": "#/definitions/Expr"
            },
            {
              "type": "null"
            }
          ]
        },
        "format": {
          "type": "string",
          "enum": [
            "tsv"
          ]
        },
        "inline": {
          "anyOf": [
            {
              "$ref": "#/definitions/Expr"
            },
            {
              "type": "null"
            }
          ]
        },
        "offset": {
          "type": [
            "integer",
            "null"
          ],
          "format": "uint",
          "minimum": 0.0
        }
      }
    },
    {
      "title": "JSON",
      "type": "object",
      "required": [
        "format"
      ],
      "properties": {
        "dataset": {
          "anyOf": [
            {
              "$ref": "#/definitions/Expr"
            },
            {
              "type": "null"
            }
          ]
        },
        "format": {
          "type": "string",
          "enum": [
            "json"
          ]
        },
        "inline": {
          "anyOf": [
            {
              "$ref": "#/definitions/Expr"
            },
            {
              "type": "null"
            }
          ]
        }
      }
    },
    {
      "title": "CityGML",
      "type": "object",
      "required": [
        "format"
      ],
      "properties": {
        "dataset": {
          "anyOf": [
            {
              "$ref": "#/definitions/Expr"
            },
            {
              "type": "null"
            }
          ]
        },
        "flatten": {
          "type": [
            "boolean",
            "null"
          ]
        },
        "format": {
          "type": "string",
          "enum": [
            "citygml"
          ]
        },
        "inline": {
          "anyOf": [
            {
              "$ref": "#/definitions/Expr"
            },
            {
              "type": "null"
            }
          ]
        }
      }
    },
    {
      "title": "GeoJSON",
      "type": "object",
      "required": [
        "format"
      ],
      "properties": {
        "dataset": {
          "anyOf": [
            {
              "$ref": "#/definitions/Expr"
            },
            {
              "type": "null"
            }
          ]
        },
        "format": {
          "type": "string",
          "enum": [
            "geojson"
          ]
        },
        "inline": {
          "anyOf": [
            {
              "$ref": "#/definitions/Expr"
            },
            {
              "type": "null"
            }
          ]
        }
      }
    }
  ],
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FileWriterParam",
  "oneOf": [
    {
      "type": "object",
      "required": [
        "format",
        "output"
      ],
      "properties": {
        "format": {
          "type": "string",
          "enum": [
            "csv"
          ]
        },
        "output": {
          "$ref": "#/definitions/Expr"
        }
      }
    },
    {
      "type": "object",
      "required": [
        "format",
        "output"
      ],
      "properties": {
        "format": {
          "type": "string",
          "enum": [
            "tsv"
          ]
        },
        "output": {
          "$ref": "#/definitions/Expr"
        }
      }
    },
    {
      "type": "object",
      "required": [
        "format",
        "output"
      ],
      "properties": {
        "converter": {
          "anyOf": [
            {
              "$ref": "#/definitions/Expr"
            },
            {
              "type": "null"
            }
          ]
        },
        "format": {
          "type": "string",
          "enum": [
            "json"
          ]
        },
        "output": {
          "$ref": "#/definitions/Expr"
        }
      }
    },
    {
      "type": "object",
      "required": [
        "format",
        "output"
      ],
      "properties": {
        "format": {
          "type": "string",
          "enum": [
            "excel"
          ]
        },
        "output": {
          "$ref": "#/definitions/Expr"
        },
        "sheetName": {
          "type": [
            "string",
            "null"
          ]
        }
      }
    }
  ],
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GeoJsonWriterParam",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "groupBy": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "output": {
      "$ref": "#/definitions/Expr"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GeometryCoercer",
  "type": "object",
  "required": [
    "coercerType"
  ],
  "properties": {
    "coercerType": {
      "$ref": "#/definitions/CoercerType"
    }
  },
  "definitions": {
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

## GeometryExtractor
### Type
* processor
### Description
Extracts geometry from a feature and adds it as an attribute.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GeometryExtractor",
  "type": "object",
  "required": [
    "outputAttribute"
  ],
  "properties": {
    "outputAttribute": {
      "$ref": "#/definitions/Attribute"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GeometryFilterParam",
  "oneOf": [
    {
      "type": "object",
      "required": [
        "filterType"
      ],
      "properties": {
        "filterType": {
          "type": "string",
          "enum": [
            "none"
          ]
        }
      }
    },
    {
      "type": "object",
      "required": [
        "filterType"
      ],
      "properties": {
        "filterType": {
          "type": "string",
          "enum": [
            "multiple"
          ]
        }
      }
    },
    {
      "type": "object",
      "required": [
        "filterType"
      ],
      "properties": {
        "filterType": {
          "type": "string",
          "enum": [
            "geometryType"
          ]
        }
      }
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

## GeometryReplacer
### Type
* processor
### Description
Replaces the geometry of a feature with a new geometry.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GeometryReplacer",
  "type": "object",
  "required": [
    "sourceAttribute"
  ],
  "properties": {
    "sourceAttribute": {
      "$ref": "#/definitions/Attribute"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GeometryValidator",
  "type": "object",
  "required": [
    "validationTypes"
  ],
  "properties": {
    "validationTypes": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/ValidationType"
      }
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GltfWriterParam",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "attachTexture": {
      "type": [
        "boolean",
        "null"
      ]
    },
    "output": {
      "$ref": "#/definitions/Expr"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "HoleCounterParam",
  "type": "object",
  "required": [
    "outputAttribute"
  ],
  "properties": {
    "outputAttribute": {
      "$ref": "#/definitions/Attribute"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "HorizontalReprojectorParam",
  "type": "object",
  "properties": {
    "epsgCode": {
      "type": [
        "integer",
        "null"
      ],
      "format": "uint16",
      "minimum": 0.0
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "InputRouter",
  "type": "object",
  "required": [
    "routingPort"
  ],
  "properties": {
    "routingPort": {
      "type": "string"
    }
  }
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "LineOnLineOverlayerParam",
  "type": "object",
  "required": [
    "outputAttribute"
  ],
  "properties": {
    "groupBy": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "outputAttribute": {
      "$ref": "#/definitions/Attribute"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ListExploder",
  "type": "object",
  "required": [
    "sourceAttribute"
  ],
  "properties": {
    "sourceAttribute": {
      "$ref": "#/definitions/Attribute"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "MVTWriterParam",
  "type": "object",
  "required": [
    "layerName",
    "maxZoom",
    "minZoom",
    "output"
  ],
  "properties": {
    "compressOutput": {
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "layerName": {
      "$ref": "#/definitions/Expr"
    },
    "maxZoom": {
      "type": "integer",
      "format": "uint8",
      "minimum": 0.0
    },
    "minZoom": {
      "type": "integer",
      "format": "uint8",
      "minimum": 0.0
    },
    "output": {
      "$ref": "#/definitions/Expr"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "OrientationExtractorParam",
  "type": "object",
  "required": [
    "outputAttribute"
  ],
  "properties": {
    "outputAttribute": {
      "$ref": "#/definitions/Attribute"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "OutputRouter",
  "type": "object",
  "required": [
    "routingPort"
  ],
  "properties": {
    "routingPort": {
      "type": "string"
    }
  }
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

## PLATEAU4.AttributeFlattener
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

## PLATEAU4.CityCodeExtractor
### Type
* processor
### Description
Extracts Codelist
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CityCodeExtractorParam",
  "type": "object",
  "required": [
    "cityCodeAttribute",
    "codelistsPathAttribute"
  ],
  "properties": {
    "cityCodeAttribute": {
      "$ref": "#/definitions/Attribute"
    },
    "codelistsPathAttribute": {
      "$ref": "#/definitions/Attribute"
    }
  },
  "definitions": {
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
* PLATEAU

## PLATEAU4.MaxLodExtractor
### Type
* processor
### Description
Extracts maxLod
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "MaxLodExtractorParam",
  "type": "object",
  "required": [
    "cityGmlPathAttribute",
    "maxLodAttribute"
  ],
  "properties": {
    "cityGmlPathAttribute": {
      "$ref": "#/definitions/Attribute"
    },
    "maxLodAttribute": {
      "$ref": "#/definitions/Attribute"
    }
  },
  "definitions": {
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
* PLATEAU

## PLATEAU4.MissingAttributeDetector
### Type
* processor
### Description
Detect missing attributes
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "MissingAttributeDetectorParam",
  "type": "object",
  "required": [
    "packageAttribute"
  ],
  "properties": {
    "packageAttribute": {
      "$ref": "#/definitions/Attribute"
    }
  },
  "definitions": {
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* summary
* required
* target
### Category
* PLATEAU

## PLATEAU4.ObjectListExtractor
### Type
* processor
### Description
Extract object list
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ObjectListExtractorParam",
  "type": "object",
  "required": [
    "objectListPathAttribute"
  ],
  "properties": {
    "objectListPathAttribute": {
      "$ref": "#/definitions/Attribute"
    }
  },
  "definitions": {
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
* PLATEAU

## PLATEAU4.UDXFolderExtractor
### Type
* processor
### Description
Extracts UDX folders from cityGML path
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "UDXFolderExtractorParam",
  "type": "object",
  "required": [
    "cityGmlPath"
  ],
  "properties": {
    "cityGmlPath": {
      "$ref": "#/definitions/Expr"
    },
    "codelistsPath": {
      "anyOf": [
        {
          "$ref": "#/definitions/Attribute"
        },
        {
          "type": "null"
        }
      ]
    },
    "schemasPath": {
      "anyOf": [
        {
          "$ref": "#/definitions/Attribute"
        },
        {
          "type": "null"
        }
      ]
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "RhaiCallerParam",
  "type": "object",
  "required": [
    "isTarget",
    "process"
  ],
  "properties": {
    "isTarget": {
      "$ref": "#/definitions/Expr"
    },
    "process": {
      "$ref": "#/definitions/Expr"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "StatisticsCalculatorParam",
  "type": "object",
  "required": [
    "calculations"
  ],
  "properties": {
    "aggregateAttribute": {
      "anyOf": [
        {
          "$ref": "#/definitions/Attribute"
        },
        {
          "type": "null"
        }
      ]
    },
    "aggregateName": {
      "anyOf": [
        {
          "$ref": "#/definitions/Attribute"
        },
        {
          "type": "null"
        }
      ]
    },
    "calculations": {
      "type": "array",
      "items": {
        "$ref": "#/definitions/Calculation"
      }
    }
  },
  "definitions": {
    "Attribute": {
      "type": "string"
    },
    "Calculation": {
      "type": "object",
      "required": [
        "expr",
        "newAttribute"
      ],
      "properties": {
        "expr": {
          "$ref": "#/definitions/Expr"
        },
        "newAttribute": {
          "$ref": "#/definitions/Attribute"
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ThreeDimensionBoxReplacer",
  "type": "object",
  "required": [
    "maxX",
    "maxY",
    "maxZ",
    "minX",
    "minY",
    "minZ"
  ],
  "properties": {
    "maxX": {
      "$ref": "#/definitions/Attribute"
    },
    "maxY": {
      "$ref": "#/definitions/Attribute"
    },
    "maxZ": {
      "$ref": "#/definitions/Attribute"
    },
    "minX": {
      "$ref": "#/definitions/Attribute"
    },
    "minY": {
      "$ref": "#/definitions/Attribute"
    },
    "minZ": {
      "$ref": "#/definitions/Attribute"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ThreeDimensionRotatorParam",
  "type": "object",
  "required": [
    "angleDegree",
    "directionX",
    "directionY",
    "directionZ",
    "originX",
    "originY",
    "originZ"
  ],
  "properties": {
    "angleDegree": {
      "$ref": "#/definitions/Expr"
    },
    "directionX": {
      "$ref": "#/definitions/Expr"
    },
    "directionY": {
      "$ref": "#/definitions/Expr"
    },
    "directionZ": {
      "$ref": "#/definitions/Expr"
    },
    "originX": {
      "$ref": "#/definitions/Expr"
    },
    "originY": {
      "$ref": "#/definitions/Expr"
    },
    "originZ": {
      "$ref": "#/definitions/Expr"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "VerticalReprojectorParam",
  "type": "object",
  "required": [
    "reprojectorType"
  ],
  "properties": {
    "reprojectorType": {
      "$ref": "#/definitions/VerticalReprojectorType"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "WasmRuntimeExecutorParam",
  "type": "object",
  "required": [
    "processorType",
    "programmingLanguage",
    "sourceCodeFilePath"
  ],
  "properties": {
    "processorType": {
      "$ref": "#/definitions/ProcessorType"
    },
    "programmingLanguage": {
      "$ref": "#/definitions/ProgrammingLanguage"
    },
    "sourceCodeFilePath": {
      "type": "string"
    }
  },
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "XmlFragmenterParam",
  "oneOf": [
    {
      "type": "object",
      "required": [
        "attribute",
        "elementsToExclude",
        "elementsToMatch",
        "source"
      ],
      "properties": {
        "attribute": {
          "$ref": "#/definitions/Attribute"
        },
        "elementsToExclude": {
          "$ref": "#/definitions/Expr"
        },
        "elementsToMatch": {
          "$ref": "#/definitions/Expr"
        },
        "source": {
          "type": "string",
          "enum": [
            "url"
          ]
        }
      }
    }
  ],
  "definitions": {
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
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "XmlValidatorParam",
  "type": "object",
  "required": [
    "attribute",
    "inputType",
    "validationType"
  ],
  "properties": {
    "attribute": {
      "$ref": "#/definitions/Attribute"
    },
    "inputType": {
      "$ref": "#/definitions/XmlInputType"
    },
    "validationType": {
      "$ref": "#/definitions/ValidationType"
    }
  },
  "definitions": {
    "Attribute": {
      "type": "string"
    },
    "ValidationType": {
      "type": "string",
      "enum": [
        "syntax",
        "syntaxAndNamespace",
        "syntaxAndSchema"
      ]
    },
    "XmlInputType": {
      "type": "string",
      "enum": [
        "file",
        "text"
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
