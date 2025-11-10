# Actions

## AreaOnAreaOverlayer
### Type
* processor
### Description
Perform Area Overlay Analysis
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AreaOnAreaOverlayer Parameters",
  "description": "Configure how area overlay analysis is performed",
  "type": "object",
  "properties": {
    "accumulationMode": {
      "title": "Accumulation Mode",
      "description": "Controls how attributes from input features are handled in output features",
      "default": "useAttributesFromOneFeature",
      "allOf": [
        {
          "$ref": "#/definitions/AccumulationMode"
        }
      ]
    },
    "generateList": {
      "title": "Generate List",
      "description": "Name of the list attribute to store source feature attributes",
      "type": [
        "string",
        "null"
      ]
    },
    "groupBy": {
      "title": "Group By Attributes",
      "description": "Optional attributes to group features by during overlay analysis",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "outputAttribute": {
      "title": "Output Attribute",
      "description": "Name of the attribute to store overlap count",
      "type": [
        "string",
        "null"
      ]
    },
    "tolerance": {
      "title": "Tolerance",
      "description": "Geometric tolerance. Vertices closer than this distance will be considered identical during the overlay operation.",
      "type": [
        "number",
        "null"
      ],
      "format": "double"
    }
  },
  "definitions": {
    "AccumulationMode": {
      "type": "string",
      "enum": [
        "useAttributesFromOneFeature",
        "dropIncomingAttributes"
      ]
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
* area
* remnants
* rejected
### Category
* Geometry

## AttributeAggregator
### Type
* processor
### Description
Group and Aggregate Features by Attributes
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AttributeAggregator Parameters",
  "description": "Configure how features are grouped and aggregated based on attribute values",
  "type": "object",
  "required": [
    "aggregateAttributes",
    "calculationAttribute",
    "method"
  ],
  "properties": {
    "aggregateAttributes": {
      "title": "List of attributes to aggregate",
      "type": "array",
      "items": {
        "$ref": "#/definitions/AggregateAttribute"
      }
    },
    "calculation": {
      "title": "Calculation to perform",
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
      "title": "Attribute to store calculation result",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "calculationValue": {
      "title": "Value to use for calculation",
      "type": [
        "integer",
        "null"
      ],
      "format": "int64"
    },
    "method": {
      "title": "Method to use for aggregation",
      "allOf": [
        {
          "$ref": "#/definitions/Method"
        }
      ]
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
          "title": "Existing attribute to use",
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
          "title": "Value to use for attribute",
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
          "title": "New attribute to create",
          "allOf": [
            {
              "$ref": "#/definitions/Attribute"
            }
          ]
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
      "oneOf": [
        {
          "title": "Maximum Value",
          "description": "Find the maximum value in the group",
          "type": "string",
          "enum": [
            "max"
          ]
        },
        {
          "title": "Minimum Value",
          "description": "Find the minimum value in the group",
          "type": "string",
          "enum": [
            "min"
          ]
        },
        {
          "title": "Count Items",
          "description": "Count the number of features in the group",
          "type": "string",
          "enum": [
            "count"
          ]
        }
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
Join Array Attributes Into Single Values
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AttributeBulkArrayJoiner Parameters",
  "description": "Configure which array attributes to join into single values",
  "type": "object",
  "properties": {
    "ignoreAttributes": {
      "title": "Attributes to Ignore",
      "description": "List of attribute names to skip during array joining process",
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

## AttributeConversionTable
### Type
* processor
### Description
Transform Feature Attributes Using Lookup Tables
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AttributeConversionTable Parameters",
  "type": "object",
  "required": [
    "format",
    "rules"
  ],
  "properties": {
    "dataset": {
      "title": "Dataset URI",
      "description": "Path or URI to external conversion table file",
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
      "title": "Table Format",
      "description": "Format of the conversion table (CSV, TSV, or JSON)",
      "allOf": [
        {
          "$ref": "#/definitions/ConversionTableFormat"
        }
      ]
    },
    "inline": {
      "title": "Inline Table Data",
      "description": "Conversion table data provided directly as string content",
      "type": [
        "string",
        "null"
      ]
    },
    "rules": {
      "title": "Conversion Rules",
      "description": "List of rules defining how to map attributes using the conversion table",
      "type": "array",
      "items": {
        "$ref": "#/definitions/AttributeConversionTableRule"
      }
    }
  },
  "definitions": {
    "Attribute": {
      "type": "string"
    },
    "AttributeConversionTableRule": {
      "type": "object",
      "required": [
        "conversionTableKeys",
        "conversionTableTo",
        "featureFroms",
        "featureTo"
      ],
      "properties": {
        "conversionTableKeys": {
          "title": "Keys to match in conversion table",
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "conversionTableTo": {
          "title": "Attribute to convert to",
          "type": "string"
        },
        "featureFroms": {
          "title": "Attributes to convert from",
          "type": "array",
          "items": {
            "$ref": "#/definitions/Attribute"
          }
        },
        "featureTo": {
          "title": "Attribute to convert to",
          "allOf": [
            {
              "$ref": "#/definitions/Attribute"
            }
          ]
        }
      }
    },
    "ConversionTableFormat": {
      "type": "string",
      "enum": [
        "csv",
        "tsv",
        "json"
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

## AttributeDuplicateFilter
### Type
* processor
### Description
Remove Duplicate Features Based on Attribute Values
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AttributeDuplicateFilter Parameters",
  "type": "object",
  "required": [
    "filterBy"
  ],
  "properties": {
    "filterBy": {
      "title": "Filter Attributes",
      "description": "Attributes used to identify duplicate features - features with identical values for these attributes will be deduplicated",
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
Extract File System Information from Path Attributes
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AttributeFilePathInfoExtractor Parameters",
  "type": "object",
  "required": [
    "attribute"
  ],
  "properties": {
    "attribute": {
      "title": "Source Path Attribute",
      "description": "Attribute containing the file path to analyze for extracting file system information",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
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
Flatten Nested Object Attributes into Top-Level Attributes
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AttributeFlattener Parameters",
  "type": "object",
  "required": [
    "attributes"
  ],
  "properties": {
    "attributes": {
      "title": "Attributes to Flatten",
      "description": "Map/object attributes that should be flattened - their nested properties will become top-level attributes",
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
Create, Convert, Rename, and Remove Feature Attributes
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AttributeManager Parameters",
  "type": "object",
  "required": [
    "operations"
  ],
  "properties": {
    "operations": {
      "title": "Attribute Operations",
      "description": "List of operations to perform on feature attributes (create, convert, rename, remove)",
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
          "title": "Attribute name",
          "type": "string"
        },
        "method": {
          "title": "Operation to perform",
          "allOf": [
            {
              "$ref": "#/definitions/Method"
            }
          ]
        },
        "value": {
          "title": "Value",
          "description": "Value to use for the operation",
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
Transform Feature Attributes Using Expressions and Mappings
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AttributeMapper Parameters",
  "type": "object",
  "required": [
    "mappers"
  ],
  "properties": {
    "mappers": {
      "title": "Attribute Mappers",
      "description": "List of mapping rules to transform attributes using expressions or value copying",
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
          "title": "Attribute name",
          "type": [
            "string",
            "null"
          ]
        },
        "childAttribute": {
          "title": "Child attribute name",
          "type": [
            "string",
            "null"
          ]
        },
        "expr": {
          "title": "Expression to evaluate",
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
          "title": "Expression to evaluate multiple attributes",
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
          "title": "Parent attribute name",
          "type": [
            "string",
            "null"
          ]
        },
        "valueAttribute": {
          "title": "Attribute name to get value from",
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
Extract Bounding Box Coordinates from Feature Geometry
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "BoundsExtractor Parameters",
  "type": "object",
  "properties": {
    "xmax": {
      "title": "Maximum X Attribute",
      "description": "Attribute name for storing the maximum X coordinate (defaults to \"xmax\")",
      "anyOf": [
        {
          "$ref": "#/definitions/Attribute"
        },
        {
          "type": "null"
        }
      ]
    },
    "xmin": {
      "title": "Minimum X Attribute",
      "description": "Attribute name for storing the minimum X coordinate (defaults to \"xmin\")",
      "anyOf": [
        {
          "$ref": "#/definitions/Attribute"
        },
        {
          "type": "null"
        }
      ]
    },
    "ymax": {
      "title": "Maximum Y Attribute",
      "description": "Attribute name for storing the maximum Y coordinate (defaults to \"ymax\")",
      "anyOf": [
        {
          "$ref": "#/definitions/Attribute"
        },
        {
          "type": "null"
        }
      ]
    },
    "ymin": {
      "title": "Minimum Y Attribute",
      "description": "Attribute name for storing the minimum Y coordinate (defaults to \"ymin\")",
      "anyOf": [
        {
          "$ref": "#/definitions/Attribute"
        },
        {
          "type": "null"
        }
      ]
    },
    "zmax": {
      "title": "Maximum Z Attribute",
      "description": "Attribute name for storing the maximum Z coordinate (defaults to \"zmax\")",
      "anyOf": [
        {
          "$ref": "#/definitions/Attribute"
        },
        {
          "type": "null"
        }
      ]
    },
    "zmin": {
      "title": "Minimum Z Attribute",
      "description": "Attribute name for storing the minimum Z coordinate (defaults to \"zmin\")",
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

## Bufferer
### Type
* processor
### Description
Create Buffer Around Features
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Bufferer Parameters",
  "description": "Configure how to create buffers around input geometries",
  "type": "object",
  "required": [
    "bufferType",
    "distance",
    "interpolationAngle"
  ],
  "properties": {
    "bufferType": {
      "title": "Buffer Type",
      "description": "The type of buffer to create around the input geometry",
      "allOf": [
        {
          "$ref": "#/definitions/BufferType"
        }
      ]
    },
    "distance": {
      "title": "Distance",
      "description": "The distance to extend the buffer from the original geometry (in coordinate units)",
      "type": "number",
      "format": "double"
    },
    "interpolationAngle": {
      "title": "Interpolation Angle",
      "description": "The angle in degrees used for curve interpolation when creating rounded corners",
      "type": "number",
      "format": "double"
    }
  },
  "definitions": {
    "BufferType": {
      "oneOf": [
        {
          "title": "2D Area Buffer",
          "description": "Creates a 2D polygon buffer around the input geometry",
          "type": "string",
          "enum": [
            "area2d"
          ]
        }
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
Rename Feature Attributes in Bulk
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "BulkAttributeRenamer Parameters",
  "description": "Configure how to rename feature attributes in bulk operations",
  "type": "object",
  "required": [
    "renameAction",
    "renameType",
    "renameValue"
  ],
  "properties": {
    "renameAction": {
      "title": "Rename Operation",
      "description": "The type of renaming operation to perform on the attribute names",
      "allOf": [
        {
          "$ref": "#/definitions/RenameAction"
        }
      ]
    },
    "renameType": {
      "title": "Which Attributes to Rename",
      "description": "Choose whether to rename all attributes or only selected ones",
      "allOf": [
        {
          "$ref": "#/definitions/RenameType"
        }
      ]
    },
    "renameValue": {
      "title": "Text Value",
      "description": "The text to add as prefix/suffix, remove, or use as replacement",
      "type": "string"
    },
    "selectedAttributes": {
      "title": "Selected Attribute Names",
      "description": "List of specific attribute names to rename (required when \"Selected Attributes\" is chosen)",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "type": "string"
      }
    },
    "textToFind": {
      "title": "Text Pattern to Find",
      "description": "Regular expression pattern to match when using \"Replace Text\" operation",
      "type": [
        "string",
        "null"
      ]
    }
  },
  "definitions": {
    "RenameAction": {
      "oneOf": [
        {
          "title": "Add Prefix",
          "description": "Add text to the beginning of attribute names",
          "type": "string",
          "enum": [
            "AddPrefix"
          ]
        },
        {
          "title": "Add Suffix",
          "description": "Add text to the end of attribute names",
          "type": "string",
          "enum": [
            "AddSuffix"
          ]
        },
        {
          "title": "Remove Prefix",
          "description": "Remove text from the beginning of attribute names",
          "type": "string",
          "enum": [
            "RemovePrefix"
          ]
        },
        {
          "title": "Remove Suffix",
          "description": "Remove text from the end of attribute names",
          "type": "string",
          "enum": [
            "RemoveSuffix"
          ]
        },
        {
          "title": "Replace Text",
          "description": "Find and replace text using regular expressions",
          "type": "string",
          "enum": [
            "StringReplace"
          ]
        }
      ]
    },
    "RenameType": {
      "oneOf": [
        {
          "title": "All Attributes",
          "description": "Rename all attributes in the feature",
          "type": "string",
          "enum": [
            "All"
          ]
        },
        {
          "title": "Selected Attributes",
          "description": "Rename only specific attributes listed below",
          "type": "string",
          "enum": [
            "Selected"
          ]
        }
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

## CSGBuilder
### Type
* processor
### Description
Constructs a Consecutive Solid Geometry (CSG) representation from a pair (Left, Right) of solid geometries. It detects union, intersection, difference (Left - Right). It however does not compute the resulting geometry, but outputs the CSG tree structure. To evaluate the CSG tree into a solid geometry, use CSGEvaluator.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CSG Builder Parameters",
  "description": "Configure how the CSG builder pairs features from left and right ports",
  "type": "object",
  "properties": {
    "createList": {
      "title": "Create List",
      "description": "When enabled, creates a list of attribute values from both children (left and right)",
      "type": [
        "boolean",
        "null"
      ]
    },
    "listAttributeName": {
      "title": "List Attribute Name",
      "description": "Name of the attribute to create the list from (required when create_list is true)",
      "type": [
        "string",
        "null"
      ]
    },
    "pairIdAttribute": {
      "title": "Pair ID Attribute",
      "description": "Expression to evaluate the pair ID used to match features from left and right ports",
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
    "Expr": {
      "type": "string"
    }
  }
}
```
### Input Ports
* left
* right
### Output Ports
* intersection
* union
* difference
* rejected
### Category
* Geometry

## CSGEvaluator
### Type
* processor
### Description
Evaluates a Constructive Solid Geometry (CSG) tree to produce a solid geometry. Takes a CSG representation and computes the resulting mesh from the boolean operations.
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
* nullport
* rejected
### Category
* Geometry

## CenterPointReplacer
### Type
* processor
### Description
Replace Feature Geometry with Center Point
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
Export Features as Cesium 3D Tiles for Web Visualization
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Cesium3DTilesWriter Parameters",
  "type": "object",
  "required": [
    "maxZoom",
    "minZoom",
    "output"
  ],
  "properties": {
    "attachTexture": {
      "title": "Attach Textures",
      "description": "Whether to include texture information in the generated tiles",
      "type": [
        "boolean",
        "null"
      ]
    },
    "compressOutput": {
      "title": "Compressed Output Path",
      "description": "Optional path for compressed archive output",
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "dracoCompression": {
      "type": [
        "boolean",
        "null"
      ]
    },
    "maxZoom": {
      "title": "Maximum Zoom Level",
      "description": "Maximum zoom level for tile generation (0-24)",
      "type": "integer",
      "format": "uint8",
      "minimum": 0.0
    },
    "minZoom": {
      "title": "Minimum Zoom Level",
      "description": "Minimum zoom level for tile generation (0-24)",
      "type": "integer",
      "format": "uint8",
      "minimum": 0.0
    },
    "output": {
      "title": "Output Path",
      "description": "Directory path where the 3D tiles will be written",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
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
* schema
### Output Ports
### Category
* File

## CityGmlReader
### Type
* source
### Description
Reads 3D city models from CityGML files.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CityGmlReader Parameters",
  "description": "Configuration for reading CityGML files as 3D city models.",
  "type": "object",
  "properties": {
    "dataset": {
      "title": "File Path",
      "description": "Expression that returns the path to the input file (e.g., \"data.csv\" or variable reference)",
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
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
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

## Clipper
### Type
* processor
### Description
Clip Features Using Boundary Shapes
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
Filter LineString Features by Closed/Open Status
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
Generate Convex Hull Polygons from Grouped Features
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ConvexHullAccumulator Parameters",
  "type": "object",
  "properties": {
    "groupBy": {
      "title": "Group By Attributes",
      "description": "Attributes used to group features before creating convex hulls - each group gets its own hull",
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

## CsvReader
### Type
* source
### Description
Read Features from CSV or TSV File
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CsvReader Parameters",
  "description": "Configure how CSV and TSV files are processed and read",
  "type": "object",
  "required": [
    "format"
  ],
  "properties": {
    "dataset": {
      "title": "File Path",
      "description": "Expression that returns the path to the input file (e.g., \"data.csv\" or variable reference)",
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
      "title": "File Format",
      "description": "Choose the delimiter format for the input file",
      "allOf": [
        {
          "$ref": "#/definitions/CsvFormat"
        }
      ]
    },
    "geometry": {
      "title": "Geometry Configuration",
      "description": "Optional configuration for parsing geometry from CSV columns",
      "anyOf": [
        {
          "$ref": "#/definitions/GeometryConfig"
        },
        {
          "type": "null"
        }
      ]
    },
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
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
      "title": "Header Row Offset",
      "description": "Skip this many rows from the beginning to find the header row (0 = first row is header)",
      "type": [
        "integer",
        "null"
      ],
      "format": "uint",
      "minimum": 0.0
    }
  },
  "definitions": {
    "CsvFormat": {
      "oneOf": [
        {
          "title": "CSV (Comma-Separated Values)",
          "description": "File with comma-separated values",
          "type": "string",
          "enum": [
            "csv"
          ]
        },
        {
          "title": "TSV (Tab-Separated Values)",
          "description": "File with tab-separated values",
          "type": "string",
          "enum": [
            "tsv"
          ]
        }
      ]
    },
    "Expr": {
      "type": "string"
    },
    "GeometryConfig": {
      "title": "Geometry Configuration",
      "description": "Configure how geometry data is extracted from CSV columns",
      "type": "object",
      "oneOf": [
        {
          "title": "WKT Column",
          "description": "Geometry stored as Well-Known Text in a single column",
          "type": "object",
          "required": [
            "column",
            "geometryMode"
          ],
          "properties": {
            "column": {
              "title": "WKT Column Name",
              "description": "Name of the column containing WKT geometry",
              "type": "string"
            },
            "geometryMode": {
              "type": "string",
              "enum": [
                "wkt"
              ]
            }
          }
        },
        {
          "title": "Coordinate Columns",
          "description": "Geometry stored as separate X, Y, (optional Z) columns",
          "type": "object",
          "required": [
            "geometryMode",
            "xColumn",
            "yColumn"
          ],
          "properties": {
            "geometryMode": {
              "type": "string",
              "enum": [
                "coordinates"
              ]
            },
            "xColumn": {
              "title": "X Column Name",
              "description": "Name of the column containing X coordinate (longitude)",
              "type": "string"
            },
            "yColumn": {
              "title": "Y Column Name",
              "description": "Name of the column containing Y coordinate (latitude)",
              "type": "string"
            },
            "zColumn": {
              "title": "Z Column Name",
              "description": "Optional name of the column containing Z coordinate (elevation)",
              "type": [
                "string",
                "null"
              ]
            }
          }
        }
      ],
      "properties": {
        "epsg": {
          "title": "EPSG Code",
          "description": "Coordinate Reference System code (e.g., 4326 for WGS84)",
          "type": [
            "integer",
            "null"
          ],
          "format": "uint16",
          "minimum": 0.0
        }
      }
    }
  }
}
```
### Input Ports
### Output Ports
* default
### Category
* File

## CsvWriter
### Type
* sink
### Description
Writes features to CSV or TSV files.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CsvWriter Parameters",
  "description": "Configuration for writing features to CSV/TSV files.",
  "type": "object",
  "required": [
    "format",
    "output"
  ],
  "properties": {
    "format": {
      "description": "File format: csv (comma) or tsv (tab)",
      "allOf": [
        {
          "$ref": "#/definitions/CsvFormat"
        }
      ]
    },
    "output": {
      "description": "Output path or expression for the CSV/TSV file to create",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
    }
  },
  "definitions": {
    "CsvFormat": {
      "oneOf": [
        {
          "title": "CSV (Comma-Separated Values)",
          "description": "File with comma-separated values",
          "type": "string",
          "enum": [
            "csv"
          ]
        },
        {
          "title": "TSV (Tab-Separated Values)",
          "description": "File with tab-separated values",
          "type": "string",
          "enum": [
            "tsv"
          ]
        }
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
### Category
* File

## CzmlReader
### Type
* source
### Description
Reads geographic features from CZML (Cesium Language) files for 3D visualization
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CzmlReader Parameters",
  "description": "Configuration for reading CZML files as geographic features.",
  "type": "object",
  "properties": {
    "dataset": {
      "title": "File Path",
      "description": "Expression that returns the path to the input file (e.g., \"data.csv\" or variable reference)",
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "force2d": {
      "title": "Force 2D",
      "description": "If true, forces all geometries to be 2D (ignoring Z values)",
      "default": false,
      "type": "boolean"
    },
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "skipDocumentPacket": {
      "title": "Skip Document Packet",
      "description": "If true, skips the document packet (first packet with version/clock info)",
      "default": true,
      "type": "boolean"
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

## CzmlWriter
### Type
* sink
### Description
Export Features as CZML for Cesium Visualization
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CzmlWriter Parameters",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "groupBy": {
      "title": "Group By Attributes",
      "description": "Attributes used to group features into separate CZML files",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "output": {
      "title": "Output File Path",
      "description": "Path where the CZML file will be written",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
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
### Category
* File

## DimensionFilter
### Type
* processor
### Description
Filter Features by Geometry Dimension
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
Extracts and decompresses archive files from specified attributes
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DirectoryDecompressor Parameters",
  "description": "Configures the extraction and decompression of archive files.",
  "type": "object",
  "required": [
    "archiveAttributes"
  ],
  "properties": {
    "archiveAttributes": {
      "description": "Attributes containing archive file paths to be extracted and decompressed",
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
Dissolve Features by Grouping Attributes
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Dissolver Parameters",
  "description": "Configure how to dissolve features by grouping them based on shared attributes",
  "type": "object",
  "properties": {
    "attributeAccumulation": {
      "title": "Attribute Accumulation",
      "description": "Strategy for handling attributes when dissolving features",
      "default": "useOneFeature",
      "allOf": [
        {
          "$ref": "#/definitions/AttributeAccumulationStrategy"
        }
      ]
    },
    "groupBy": {
      "title": "Group By Attributes",
      "description": "List of attribute names to group features by before dissolving. Features with the same values for these attributes will be dissolved together",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "tolerance": {
      "title": "Tolerance",
      "description": "Geometric tolerance. Vertices closer than this distance will be considered identical during the dissolve operation.",
      "type": [
        "number",
        "null"
      ],
      "format": "double"
    }
  },
  "definitions": {
    "Attribute": {
      "type": "string"
    },
    "AttributeAccumulationStrategy": {
      "title": "Attribute Accumulation Strategy",
      "description": "Defines how attributes should be handled when dissolving multiple features into one",
      "oneOf": [
        {
          "title": "Drop Incoming Attributes",
          "description": "No attributes from any incoming features will be preserved in the output (except group_by attributes if specified)",
          "type": "string",
          "enum": [
            "dropAttributes"
          ]
        },
        {
          "title": "Merge Incoming Attributes",
          "description": "The output feature will merge all input attributes. When multiple features have the same attribute with different values, all values are collected into an array",
          "type": "string",
          "enum": [
            "mergeAttributes"
          ]
        },
        {
          "title": "Use Attributes From One Feature",
          "description": "The output inherits the attributes of one representative feature (the last feature in the group)",
          "type": "string",
          "enum": [
            "useOneFeature"
          ]
        }
      ]
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
Debug Echo Features to Logs
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
Debug Echo Features to Logs
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
Extract Z-Coordinate Elevation to Attribute
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Elevation Extractor Parameters",
  "description": "Configure where to store the extracted elevation value from geometry coordinates",
  "type": "object",
  "required": [
    "outputAttribute"
  ],
  "properties": {
    "outputAttribute": {
      "title": "Output Attribute",
      "description": "Name of the attribute where the extracted elevation value will be stored",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
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

## ExcelWriter
### Type
* sink
### Description
Writes features to Microsoft Excel format (.xlsx files).
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ExcelWriter Parameters",
  "description": "Configuration for writing features to Microsoft Excel format.",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "output": {
      "description": "Output path or expression for the Excel file to create",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
    },
    "sheetName": {
      "description": "Sheet name (defaults to \"Sheet1\")",
      "type": [
        "string",
        "null"
      ]
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

## Extruder
### Type
* processor
### Description
Extrude 2D Polygons into 3D Solids
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Extruder Parameters",
  "description": "Configure how to extrude 2D polygons into 3D solid geometries",
  "type": "object",
  "required": [
    "distance"
  ],
  "properties": {
    "distance": {
      "title": "Distance",
      "description": "The vertical distance (height) to extrude the polygon. Can be a constant value or an expression",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
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

## FeatureCityGmlReader
### Type
* processor
### Description
Reads and processes features from CityGML files with optional flattening
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureCityGmlReader Parameters",
  "description": "Configuration for reading and processing CityGML files as features.",
  "type": "object",
  "required": [
    "dataset"
  ],
  "properties": {
    "dataset": {
      "title": "Dataset",
      "description": "Path or expression to the CityGML dataset file to be read",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
    },
    "flatten": {
      "title": "Flatten",
      "description": "Whether to flatten the hierarchical structure of the CityGML data",
      "type": [
        "boolean",
        "null"
      ]
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

## FeatureCounter
### Type
* processor
### Description
Count Features and Add Counter to Attribute
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Feature Counter Parameters",
  "description": "Configure how features are counted and grouped, and where to store the count",
  "type": "object",
  "required": [
    "countStart",
    "outputAttribute"
  ],
  "properties": {
    "countStart": {
      "title": "Start Count",
      "description": "Starting value for the counter",
      "type": "integer",
      "format": "int64"
    },
    "groupBy": {
      "title": "Group By Attributes",
      "description": "List of attribute names to group features by before counting",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "outputAttribute": {
      "title": "Output Attribute",
      "description": "Name of the attribute where the count will be stored",
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
Generate Custom Features Using Scripts
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureCreator Parameters",
  "description": "Configure how to generate custom features using script expressions",
  "type": "object",
  "required": [
    "creator"
  ],
  "properties": {
    "creator": {
      "title": "Script Expression",
      "description": "Write a script expression that returns a map (single feature) or array of maps (multiple features). Each map represents feature attributes as key-value pairs.",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
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
Filter Out Duplicate Features
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
Extract File Paths from Dataset to Features
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Feature File Path Extractor Parameters",
  "description": "Configure how to extract file paths from datasets and optionally extract archives",
  "type": "object",
  "required": [
    "extractArchive",
    "sourceDataset"
  ],
  "properties": {
    "destPrefix": {
      "title": "Destination Prefix",
      "description": "Optional prefix to add to extracted file paths",
      "type": [
        "string",
        "null"
      ]
    },
    "extractArchive": {
      "title": "Extract Archive",
      "description": "Whether to extract archive files found in the dataset",
      "type": "boolean"
    },
    "sourceDataset": {
      "title": "Source Dataset",
      "description": "Expression to get the source dataset path or URL",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
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
Filter Features Based on Custom Conditions
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Feature Filter Parameters",
  "description": "Configure the conditions and output ports for filtering features based on expressions",
  "type": "object",
  "required": [
    "conditions"
  ],
  "properties": {
    "conditions": {
      "title": "Filter Conditions",
      "description": "List of conditions and their corresponding output ports for routing filtered features",
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
          "title": "Condition expression",
          "allOf": [
            {
              "$ref": "#/definitions/Expr"
            }
          ]
        },
        "outputPort": {
          "title": "Output port",
          "allOf": [
            {
              "$ref": "#/definitions/Port"
            }
          ]
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
Filters features by Level of Detail (LOD), routing them to appropriate output ports
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureLodFilter Parameters",
  "description": "Configuration for filtering features based on Level of Detail (LOD).",
  "type": "object",
  "required": [
    "filterKey"
  ],
  "properties": {
    "filterKey": {
      "description": "Attribute used to group features for LOD filtering",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
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
* up_to_lod0
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
Merges requestor and supplier features based on matching attribute values
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureMerger Parameters",
  "description": "Configuration for merging requestor and supplier features based on matching attributes or expressions.",
  "type": "object",
  "properties": {
    "completeGrouped": {
      "description": "Whether to complete grouped features before processing the next group",
      "type": [
        "boolean",
        "null"
      ]
    },
    "requestorAttribute": {
      "description": "Attributes from requestor features to use for matching (alternative to requestor_attribute_value)",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "requestorAttributeValue": {
      "description": "Expression to evaluate for requestor feature matching values (alternative to requestor_attribute)",
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
      "description": "Attributes from supplier features to use for matching (alternative to supplier_attribute_value)",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "supplierAttributeValue": {
      "description": "Expression to evaluate for supplier feature matching values (alternative to supplier_attribute)",
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
Reads features from various file formats (CSV, TSV, JSON) with configurable parsing options
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureReaderParam",
  "oneOf": [
    {
      "title": "Common Reader Parameters",
      "description": "Shared configuration for all feature reader formats.",
      "type": "object",
      "required": [
        "dataset",
        "format"
      ],
      "properties": {
        "dataset": {
          "title": "Dataset",
          "description": "Path or expression to the dataset file to be read",
          "allOf": [
            {
              "$ref": "#/definitions/Expr"
            }
          ]
        },
        "format": {
          "type": "string",
          "enum": [
            "csv"
          ]
        },
        "offset": {
          "description": "The offset of the first row to read",
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
      "title": "Common Reader Parameters",
      "description": "Shared configuration for all feature reader formats.",
      "type": "object",
      "required": [
        "dataset",
        "format"
      ],
      "properties": {
        "dataset": {
          "title": "Dataset",
          "description": "Path or expression to the dataset file to be read",
          "allOf": [
            {
              "$ref": "#/definitions/Expr"
            }
          ]
        },
        "format": {
          "type": "string",
          "enum": [
            "tsv"
          ]
        },
        "offset": {
          "description": "The offset of the first row to read",
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
      "title": "Common Reader Parameters",
      "description": "Shared configuration for all feature reader formats.",
      "type": "object",
      "required": [
        "dataset",
        "format"
      ],
      "properties": {
        "dataset": {
          "title": "Dataset",
          "description": "Path or expression to the dataset file to be read",
          "allOf": [
            {
              "$ref": "#/definitions/Expr"
            }
          ]
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
Sorts features based on specified attributes in ascending or descending order
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureSorter Parameters",
  "description": "Configuration for sorting features based on attribute values.",
  "type": "object",
  "required": [
    "attributes",
    "order"
  ],
  "properties": {
    "attributes": {
      "description": "Attributes to use for sorting features (sort order based on attribute order)",
      "type": "array",
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "order": {
      "description": "Sorting order (ascending or descending)",
      "allOf": [
        {
          "$ref": "#/definitions/Order"
        }
      ]
    }
  },
  "definitions": {
    "Attribute": {
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
Applies transformation expressions to modify feature attributes and properties
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FeatureTransformer Parameters",
  "description": "Configuration for applying transformation expressions to features.",
  "type": "object",
  "required": [
    "transformers"
  ],
  "properties": {
    "transformers": {
      "description": "List of transformation expressions to apply to each feature",
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
          "description": "Expression that modifies the feature (can access and modify attributes, geometry, etc.)",
          "allOf": [
            {
              "$ref": "#/definitions/Expr"
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
  "title": "FeatureTypeFilter Parameters",
  "description": "Configuration for filtering features based on their feature type.",
  "type": "object",
  "required": [
    "targetTypes"
  ],
  "properties": {
    "targetTypes": {
      "description": "Target feature types",
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
  "title": "FeatureWriter Parameters",
  "description": "Configuration for writing features to different file formats.",
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
          "title": "Output path",
          "allOf": [
            {
              "$ref": "#/definitions/Expr"
            }
          ]
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
          "title": "Output path",
          "allOf": [
            {
              "$ref": "#/definitions/Expr"
            }
          ]
        }
      }
    },
    {
      "title": "JsonWriter Parameters",
      "description": "Configuration for writing features in JSON format with optional custom conversion.",
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
          "title": "Output path",
          "allOf": [
            {
              "$ref": "#/definitions/Expr"
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
* default
### Output Ports
* default
### Category
* Feature

## FilePathExtractor
### Type
* source
### Description
Extracts file paths from directories or archives, creating features for each discovered file
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FilePathExtractor Parameters",
  "description": "Configuration for extracting file paths from directories or archives.",
  "type": "object",
  "required": [
    "extractArchive",
    "sourceDataset"
  ],
  "properties": {
    "extractArchive": {
      "title": "Extract Archive",
      "description": "Whether to extract files from archives (zip files, etc.) or just list them",
      "type": "boolean"
    },
    "sourceDataset": {
      "title": "Source Dataset",
      "description": "Path or expression pointing to the source directory or archive file",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
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
Extracts file system properties (type, size, timestamps) from files
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FilePropertyExtractor Parameters",
  "description": "Configuration for extracting file system properties from files.",
  "type": "object",
  "required": [
    "filePathAttribute"
  ],
  "properties": {
    "filePathAttribute": {
      "description": "Attribute name containing the file path to analyze for properties",
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

## GeoJsonReader
### Type
* source
### Description
Reads geographic features from GeoJSON files, supporting both single features and feature collections
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GeoJsonReader Parameters",
  "description": "Configuration for reading GeoJSON files as geographic features.",
  "type": "object",
  "properties": {
    "dataset": {
      "title": "File Path",
      "description": "Expression that returns the path to the input file (e.g., \"data.csv\" or variable reference)",
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
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

## GeoJsonWriter
### Type
* sink
### Description
Writes geographic features to GeoJSON files with optional grouping
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GeoJsonWriter Parameters",
  "description": "Configuration for writing features to GeoJSON files.",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "groupBy": {
      "description": "Optional attributes to group features by, creating separate files for each group",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "output": {
      "description": "Output path or expression for the GeoJSON file to create",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
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
### Category
* File

## GeoPackageReader
### Type
* source
### Description
Reads geographic features from GeoPackage (.gpkg) files with support for vector features, tiles, and metadata
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GeoPackageReaderParam",
  "type": "object",
  "properties": {
    "attributeFilter": {
      "default": null,
      "type": [
        "string",
        "null"
      ]
    },
    "batchSize": {
      "default": null,
      "type": [
        "integer",
        "null"
      ],
      "format": "uint",
      "minimum": 0.0
    },
    "dataset": {
      "title": "File Path",
      "description": "Expression that returns the path to the input file (e.g., \"data.csv\" or variable reference)",
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "force2D": {
      "default": false,
      "type": "boolean"
    },
    "includeMetadata": {
      "default": false,
      "type": "boolean"
    },
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
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
      "type": [
        "string",
        "null"
      ]
    },
    "readMode": {
      "default": "features",
      "allOf": [
        {
          "$ref": "#/definitions/GeoPackageReadMode"
        }
      ]
    },
    "spatialFilter": {
      "default": null,
      "type": [
        "string",
        "null"
      ]
    },
    "tileFormat": {
      "default": "png",
      "allOf": [
        {
          "$ref": "#/definitions/TileFormat"
        }
      ]
    }
  },
  "definitions": {
    "Expr": {
      "type": "string"
    },
    "GeoPackageReadMode": {
      "type": "string",
      "enum": [
        "features",
        "tiles",
        "all",
        "metadataOnly"
      ]
    },
    "TileFormat": {
      "type": "string",
      "enum": [
        "png",
        "jpeg",
        "webp"
      ]
    }
  }
}
```
### Input Ports
### Output Ports
* default
### Category
* File
* Database

## GeometryCoercer
### Type
* processor
### Description
Coerces and converts feature geometries to specified target geometry types
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GeometryCoercer Parameters",
  "description": "Configuration for coercing geometries to specific target types.",
  "type": "object",
  "required": [
    "coercerType"
  ],
  "properties": {
    "coercerType": {
      "description": "Target geometry type to coerce features to (e.g., LineString)",
      "allOf": [
        {
          "$ref": "#/definitions/CoercerType"
        }
      ]
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
Extract Geometry Data to Attribute
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Geometry Extractor Parameters",
  "description": "Configure where to store the extracted geometry data as a compressed attribute",
  "type": "object",
  "required": [
    "outputAttribute"
  ],
  "properties": {
    "outputAttribute": {
      "title": "Output Attribute",
      "description": "Name of the attribute where the extracted geometry data will be stored as compressed JSON",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
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
Filter Features by Geometry Type
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Geometry Filter Parameters",
  "description": "Configure how to filter features based on their geometry type",
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

## GeometryPartExtractor
### Type
* processor
### Description
Extract geometry parts (surfaces) from 3D geometries as separate features
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Geometry Part Extractor Parameters",
  "description": "Configure which geometry parts to extract from 3D geometries",
  "type": "object",
  "properties": {
    "geometryPartType": {
      "title": "Part Type",
      "description": "Type of geometry part to extract",
      "default": "surface",
      "allOf": [
        {
          "$ref": "#/definitions/GeometryPartType"
        }
      ]
    }
  },
  "definitions": {
    "GeometryPartType": {
      "oneOf": [
        {
          "description": "Extract surfaces as separate features",
          "type": "string",
          "enum": [
            "surface"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* default
### Output Ports
* extracted
* remaining
* untouched
### Category
* Geometry

## GeometryReplacer
### Type
* processor
### Description
Replace Feature Geometry from Attribute
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Geometry Replacer Parameters",
  "description": "Configure which attribute contains the geometry data to replace the feature's current geometry",
  "type": "object",
  "required": [
    "sourceAttribute"
  ],
  "properties": {
    "sourceAttribute": {
      "title": "Source Attribute",
      "description": "Name of the attribute containing the compressed geometry data to use as the new geometry",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
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
Split Multi-Geometries into Individual Features
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
Validate Feature Geometry Quality
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Geometry Validator Parameters",
  "description": "Configure which validation checks to perform on feature geometries",
  "type": "object",
  "required": [
    "validationTypes"
  ],
  "properties": {
    "validationTypes": {
      "title": "Validation Types",
      "description": "List of validation checks to perform on the geometry (duplicate points, corrupt geometry, self-intersection)",
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
        "duplicateConsecutivePoints",
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
Filter Features by Geometry Value Type
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

## GltfReader
### Type
* source
### Description
Reads 3D models from glTF 2.0 files, supporting meshes, nodes, scenes, and geometry primitives
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GltfReaderParam",
  "type": "object",
  "properties": {
    "dataset": {
      "title": "File Path",
      "description": "Expression that returns the path to the input file (e.g., \"data.csv\" or variable reference)",
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "includeNodes": {
      "title": "Include Nodes",
      "description": "If true, includes node hierarchy information from the glTF scene graph in feature attributes",
      "default": true,
      "type": "boolean"
    },
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "mergeMeshes": {
      "title": "Merge Meshes",
      "description": "If true, combines all meshes from the glTF file into a single output feature",
      "default": false,
      "type": "boolean"
    },
    "triangulate": {
      "title": "Triangulate",
      "description": "If true, converts all primitives to triangles (reserved for future use - currently all primitives are processed as triangles)",
      "default": true,
      "type": "boolean"
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
* 3D

## GltfWriter
### Type
* sink
### Description
Writes 3D features to GLTF format with optional texture attachment
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GltfWriter Parameters",
  "description": "Configuration for writing features to GLTF 3D format.",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "attachTexture": {
      "description": "Whether to attach texture information to the GLTF model",
      "type": [
        "boolean",
        "null"
      ]
    },
    "dracoCompression": {
      "type": [
        "boolean",
        "null"
      ]
    },
    "output": {
      "description": "Output path or expression for the GLTF file to create",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
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
Count Polygon Holes to Attribute
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Hole Counter Parameters",
  "description": "Configure where to store the count of holes found in polygon geometries",
  "type": "object",
  "required": [
    "outputAttribute"
  ],
  "properties": {
    "outputAttribute": {
      "title": "Output Attribute",
      "description": "Name of the attribute where the hole count will be stored as a number",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
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
Extract Polygon Holes as Separate Features
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
Reproject Geometry to Different Coordinate System
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Horizontal Reprojector Parameters",
  "description": "Configure the target coordinate system for geometry reprojection",
  "type": "object",
  "required": [
    "epsgCode"
  ],
  "properties": {
    "epsgCode": {
      "title": "EPSG Code",
      "description": "Target coordinate system EPSG code for the reprojection",
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

## JPStandardGridAccumulator
### Type
* processor
### Description
Divides geometries into Japanese standard mesh grid (1km) and adds mesh codes to features
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
* rejected
### Category
* Geometry

## JsonReader
### Type
* source
### Description
Reads features from JSON files, supporting both single objects and arrays of objects.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "JsonReader Parameters",
  "description": "Configuration for reading JSON files as features.",
  "type": "object",
  "properties": {
    "dataset": {
      "title": "File Path",
      "description": "Expression that returns the path to the input file (e.g., \"data.csv\" or variable reference)",
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
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

## JsonWriter
### Type
* sink
### Description
Writes features to JSON files.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "JsonWriter Parameters",
  "description": "Configuration for writing features to JSON files.",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "converter": {
      "description": "Optional converter expression to transform features before writing",
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "output": {
      "description": "Output path or expression for the JSON file to create",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
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

## LineOnLineOverlayer
### Type
* processor
### Description
Intersection points are turned into point features that can contain the merged list of attributes of the original intersected lines.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "LineOnLineOverlayer Parameters",
  "description": "Configuration for finding intersection points between line features.",
  "type": "object",
  "required": [
    "tolerance"
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
    "overlaidListsAttrName": {
      "description": "Name of the attribute to store the overlaid lists. Defaults to \"overlaidLists\".",
      "type": [
        "string",
        "null"
      ]
    },
    "tolerance": {
      "type": "number",
      "format": "double"
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

## ListConcatenator
### Type
* processor
### Description
Extracts a specific attribute from each element in a list and concatenates them into a single string
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ListConcatenator Parameters",
  "description": "Configuration for concatenating a specific attribute from list elements.",
  "type": "object",
  "required": [
    "attribute",
    "list",
    "outputAttributeName",
    "separateCharacter"
  ],
  "properties": {
    "attribute": {
      "description": "Attribute name to extract from each list element",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "list": {
      "description": "List attribute to read from",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "outputAttributeName": {
      "description": "Name of the attribute to store the concatenated result",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "separateCharacter": {
      "description": "Character(s) to use as separator between concatenated values",
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
### Category
* Feature

## ListExploder
### Type
* processor
### Description
Explodes array attributes into separate features, creating one feature per array element
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ListExploder Parameters",
  "description": "Configuration for exploding array attributes into individual features.",
  "type": "object",
  "required": [
    "sourceAttribute"
  ],
  "properties": {
    "sourceAttribute": {
      "description": "Attribute containing the array to explode (each element becomes a separate feature)",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
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

## ListIndexer
### Type
* processor
### Description
Copies attributes from a specific list element to become the main attributes of a feature
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ListIndexer Parameters",
  "description": "Configuration for copying attributes from a specific list element to main feature attributes.",
  "type": "object",
  "required": [
    "listAttribute",
    "listIndexToCopy"
  ],
  "properties": {
    "copiedAttributePrefix": {
      "description": "Optional prefix to add to copied attribute names",
      "default": null,
      "type": [
        "string",
        "null"
      ]
    },
    "copiedAttributeSuffix": {
      "description": "Optional suffix to add to copied attribute names",
      "default": null,
      "type": [
        "string",
        "null"
      ]
    },
    "listAttribute": {
      "description": "List attribute to read from",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "listIndexToCopy": {
      "description": "Index of the list element to copy (0-based)",
      "type": "integer",
      "format": "uint",
      "minimum": 0.0
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
Writes vector features to Mapbox Vector Tiles (MVT) format for web mapping
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "MVTWriter Parameters",
  "description": "Configuration for writing features to Mapbox Vector Tiles (MVT) format.",
  "type": "object",
  "required": [
    "layerName",
    "maxZoom",
    "minZoom",
    "output"
  ],
  "properties": {
    "compressOutput": {
      "title": "Compress Output",
      "description": "Optional expression to determine whether to compress the output tiles",
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
      "title": "Layer Name",
      "description": "Name of the layer within the MVT tiles",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
    },
    "maxZoom": {
      "title": "Maximum Zoom",
      "description": "Maximum zoom level to generate tiles for",
      "type": "integer",
      "format": "uint8",
      "minimum": 0.0
    },
    "minZoom": {
      "title": "Minimum Zoom",
      "description": "Minimum zoom level to generate tiles for",
      "type": "integer",
      "format": "uint8",
      "minimum": 0.0
    },
    "output": {
      "title": "Output",
      "description": "Output directory path or expression for the generated MVT tiles",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
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
No-Operation Pass-Through Processor
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
No-Operation Sink (Discard Features)
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
### Category
* Noop

## ObjReader
### Type
* source
### Description
Reads 3D models from Wavefront OBJ files, supporting vertices, faces, normals, texture coordinates, and materials
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ObjReader Parameters",
  "description": "Configuration for reading Wavefront OBJ 3D model files with support for vertices, faces, normals, texture coordinates, and material definitions.",
  "type": "object",
  "properties": {
    "dataset": {
      "title": "File Path",
      "description": "Expression that returns the path to the input file (e.g., \"data.csv\" or variable reference)",
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "includeNormals": {
      "title": "Include Normals",
      "description": "Include vertex normal data in the output geometry",
      "default": true,
      "type": "boolean"
    },
    "includeTexcoords": {
      "title": "Include Texture Coordinates",
      "description": "Include texture coordinate (UV) data in the output geometry",
      "default": true,
      "type": "boolean"
    },
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "materialFile": {
      "title": "Material File",
      "description": "Expression that returns the path to an external MTL file to use instead of mtllib directives in the OBJ file. When specified, this overrides any material library references in the OBJ file.",
      "default": null,
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "mergeGroups": {
      "title": "Merge Groups",
      "description": "Merge all groups and objects into a single feature instead of creating separate features per group/object",
      "default": false,
      "type": "boolean"
    },
    "parseMaterials": {
      "title": "Parse Materials",
      "description": "Enable parsing of material definitions from MTL files referenced in the OBJ file",
      "default": true,
      "type": "boolean"
    },
    "triangulate": {
      "title": "Triangulate",
      "description": "Convert polygons with more than 3 vertices into triangles using fan triangulation",
      "default": false,
      "type": "boolean"
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
* 3D

## ObjWriter
### Type
* sink
### Description
Writes 3D features to Wavefront OBJ format with optional material (MTL) files
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "OBJ Writer Parameters",
  "description": "Configure output settings for writing 3D features to Wavefront OBJ format",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "output": {
      "title": "Output Path",
      "description": "Expression for the output file path where the OBJ file will be written",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
    },
    "writeMaterials": {
      "title": "Write Materials",
      "description": "Enable writing of material (MTL) file alongside the OBJ file",
      "default": null,
      "type": [
        "boolean",
        "null"
      ]
    },
    "writeNormals": {
      "title": "Write Normals",
      "description": "Include vertex normal vectors in the output",
      "default": null,
      "type": [
        "boolean",
        "null"
      ]
    },
    "writeTexcoords": {
      "title": "Write Texture Coordinates",
      "description": "Include texture coordinate (UV) data in the output (currently not supported - geometry types don't include UV data)",
      "default": null,
      "type": [
        "boolean",
        "null"
      ]
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
* 3D

## Offsetter
### Type
* processor
### Description
Apply Coordinate Offsets to Geometry
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Offsetter Parameters",
  "description": "Configure the X, Y, and Z coordinate offsets to apply to all geometry coordinates",
  "type": "object",
  "properties": {
    "offsetX": {
      "title": "X Offset",
      "description": "Offset to add to all X coordinates (longitude)",
      "type": [
        "number",
        "null"
      ],
      "format": "double"
    },
    "offsetY": {
      "title": "Y Offset",
      "description": "Offset to add to all Y coordinates (latitude)",
      "type": [
        "number",
        "null"
      ],
      "format": "double"
    },
    "offsetZ": {
      "title": "Z Offset",
      "description": "Offset to add to all Z coordinates (elevation)",
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
Extract Polygon Orientation to Attribute
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Orientation Extractor Parameters",
  "description": "Configure where to store the extracted polygon orientation information",
  "type": "object",
  "required": [
    "outputAttribute"
  ],
  "properties": {
    "outputAttribute": {
      "title": "Output Attribute",
      "description": "Name of the attribute where the orientation (clockwise/counter_clockwise) will be stored",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
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
Flattens hierarchical PLATEAU3 building attributes into flat structure for analysis
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
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "BuildingUsageAttributeValidatorParam",
  "type": "object",
  "properties": {
    "codelistsPath": {
      "type": [
        "string",
        "null"
      ]
    }
  }
}
```
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
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "XmlAttributeExtractorParam",
  "type": "object",
  "properties": {
    "addNsprefixToFeatureTypes": {
      "type": [
        "boolean",
        "null"
      ]
    },
    "cityCode": {
      "type": [
        "string",
        "null"
      ]
    },
    "exceptFeatureTypes": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "type": "string"
      }
    },
    "extractDmGeometryAsXmlFragment": {
      "type": [
        "boolean",
        "null"
      ]
    },
    "schemaJson": {
      "type": [
        "string",
        "null"
      ]
    },
    "targetPackages": {
      "type": [
        "array",
        "null"
      ],
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
* schema
### Category
* PLATEAU

## PLATEAU4.BuildingInstallationGeometryTypeChecker
### Type
* processor
### Description
Checks BuildingInstallation's geometry type
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
### Category
* PLATEAU

## PLATEAU4.BuildingPartConnectivityChecker
### Type
* processor
### Description
Check connectivity between BuildingParts within the same Building using 3D boundary surface matching
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "BuildingPartConnectivityChecker Parameters",
  "description": "Configure how to check connectivity between BuildingParts",
  "type": "object",
  "properties": {
    "buildingIdAttribute": {
      "title": "Building ID Attribute",
      "description": "Attribute containing the parent Building ID (default: \"gmlId\")",
      "default": "gmlId",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "fileIndexAttribute": {
      "title": "File Index Attribute",
      "description": "Attribute containing the file index (default: \"fileIndex\")",
      "default": "fileIndex",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "lodAttribute": {
      "title": "LOD Attribute",
      "description": "Attribute containing the Level of Detail (default: \"lod\")",
      "default": "lod",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "partIdAttribute": {
      "title": "Part ID Attribute",
      "description": "Attribute containing the BuildingPart ID (default: \"featureId\")",
      "default": "featureId",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
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
* PLATEAU

## PLATEAU4.BuildingUsageAttributeValidator
### Type
* processor
### Description
This processor validates building usage attributes by checking for the presence of required attributes and ensuring the correctness of city codes. It outputs errors through the lBldgError and codeError ports if any issues are found.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "BuildingUsageAttributeValidatorParam",
  "type": "object",
  "properties": {
    "codelists": {
      "type": [
        "string",
        "null"
      ]
    }
  }
}
```
### Input Ports
* default
### Output Ports
* l0405BldgError
* cityCodeError
* default
### Category
* PLATEAU

## PLATEAU4.CityCodeExtractor
### Type
* processor
### Description
Extracts city code information from PLATEAU4 codelists for local public authorities
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CityCodeExtractor Parameters",
  "description": "Configuration for extracting PLATEAU4 city code information from codelists.",
  "type": "object",
  "required": [
    "cityCodeAttribute",
    "codelistsPathAttribute"
  ],
  "properties": {
    "cityCodeAttribute": {
      "description": "Attribute containing the city code to look up in codelists",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "codelistsPathAttribute": {
      "description": "Attribute containing the path to the PLATEAU codelists directory",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
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

## PLATEAU4.DestinationMeshCodeExtractor
### Type
* processor
### Description
Extract Japanese standard regional mesh code for PLATEAU destination files and add as attribute
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "PLATEAU Destination MeshCode Extractor Parameters",
  "description": "Configure mesh code extraction for Japanese standard regional mesh",
  "type": "object",
  "properties": {
    "epsgCode": {
      "title": "EPSG Code",
      "description": "Japanese Plane Rectangular Coordinate System EPSG code for area calculation",
      "default": "6691",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
    },
    "meshType": {
      "title": "Mesh Type",
      "description": "Japanese standard mesh type: 1=80km, 2=10km, 3=1km, 4=500m, 5=250m, 6=125m",
      "default": 3,
      "type": "integer",
      "format": "uint8",
      "minimum": 0.0
    },
    "meshcodeAttr": {
      "title": "Mesh Code Attribute Name",
      "description": "Output attribute name for the mesh code",
      "default": "_meshcode",
      "type": "string"
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
* rejected
### Category
* PLATEAU

## PLATEAU4.DomainOfDefinitionValidator
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
* duplicateGmlIdStats
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
  "title": "MaxLodExtractor Parameters",
  "description": "Configuration for extracting maximum LOD (Level of Detail) information from PLATEAU4 CityGML files.",
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
  "title": "MissingAttributeDetector Parameters",
  "description": "Configuration for detecting missing attributes in PLATEAU4 features.",
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
* dataQualityC07
* dataQualityC08
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
  "title": "ObjectListExtractor Parameters",
  "description": "Configuration for extracting object lists from PLATEAU4 data.",
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

## PLATEAU4.SolidIntersectionTestPairCreator
### Type
* processor
### Description
Creates pairs of features that can possibly intersect based on bounding box overlap
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "SolidIntersectionTestPairCreatorParam",
  "type": "object",
  "properties": {
    "boundingBoxAttribute": {
      "default": "bounding_box",
      "type": "string"
    },
    "pairIdAttribute": {
      "default": "pair_id",
      "type": "string"
    }
  }
}
```
### Input Ports
* default
### Output Ports
* A
* B
### Category
* PLATEAU

## PLATEAU4.TransportationXlinkDetector
### Type
* processor
### Description
Detect unreferenced surfaces in PLATEAU transportation models (L-TRAN-03)
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "TransportationXlinkDetectorParam",
  "type": "object",
  "required": [
    "cityGmlPath"
  ],
  "properties": {
    "cityGmlPath": {
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
* passed
* failed
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
  "title": "UDXFolderExtractor Parameters",
  "description": "Configuration for extracting UDX folder structure information from PLATEAU4 CityGML paths.",
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

## PLATEAU4.UnmatchedXlinkDetector
### Type
* processor
### Description
Detect unmatched Xlinks for PLATEAU
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

## PlanarityFilter
### Type
* processor
### Description
Filter Features by Geometry Planarity
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* planarity
* notplanarity
### Category
* Geometry

## PythonScriptProcessor
### Type
* processor
### Description
Execute Python Scripts with Geospatial Data Processing
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "PythonScriptProcessorParam",
  "type": "object",
  "properties": {
    "pythonFile": {
      "title": "Python File",
      "description": "Path to a Python script file (supports file://, http://, https://, gs://, etc.)",
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "pythonPath": {
      "title": "Python Path",
      "description": "Path to Python interpreter executable (default: python3)",
      "type": [
        "string",
        "null"
      ]
    },
    "script": {
      "title": "Inline Script",
      "description": "Python script code to execute inline",
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "timeoutSeconds": {
      "title": "Timeout Seconds",
      "description": "Maximum execution time for the Python script in seconds (default: 30)",
      "type": [
        "integer",
        "null"
      ],
      "format": "uint64",
      "minimum": 0.0
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
* Script
* Python

## Refiner
### Type
* processor
### Description
Refine Complex Geometries into Simple Geometries
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
Executes Rhai script expressions to conditionally process and transform features
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "RhaiCaller Parameters",
  "description": "Configuration for executing Rhai scripts on features with conditional processing.",
  "type": "object",
  "required": [
    "isTarget",
    "process"
  ],
  "properties": {
    "isTarget": {
      "description": "Rhai script expression to determine if the feature should be processed (returns boolean)",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
    },
    "process": {
      "description": "Rhai script expression to process and transform the feature when target condition is met",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
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

## ShapefileReader
### Type
* source
### Description
Reads geographic features from Shapefile archives (.zip containing .shp, .dbf, .shx files)
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ShapefileReader Parameters",
  "description": "Configuration for reading Shapefile archives as geographic features. Expects a ZIP archive containing the required Shapefile components (.shp, .dbf, .shx).",
  "type": "object",
  "properties": {
    "dataset": {
      "title": "File Path",
      "description": "Expression that returns the path to the input file (e.g., \"data.csv\" or variable reference)",
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "encoding": {
      "title": "Character Encoding",
      "description": "Character encoding for attribute data in the DBF file (e.g., \"UTF-8\", \"Shift_JIS\")",
      "type": [
        "string",
        "null"
      ]
    },
    "force2d": {
      "title": "Force 2D",
      "description": "If true, forces all geometries to be 2D (ignoring Z values)",
      "default": false,
      "type": "boolean"
    },
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
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

## ShapefileWriter
### Type
* sink
### Description
Writes geographic features to ESRI Shapefile format with optional grouping
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ShapefileWriter Parameters",
  "description": "Configuration for writing features to ESRI Shapefile format.",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "groupBy": {
      "description": "Optional attributes to group features by, creating separate files for each group",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "output": {
      "description": "Output path or expression for the Shapefile to create",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
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
### Category
* File

## SolidBoundaryValidator
### Type
* processor
### Description
Validates the Solid Boundary Geometry
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* success
* failed
* rejected
### Category
* Geometry

## SqlReader
### Type
* source
### Description
Read Features from SQL Database
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "SQL Reader Parameters",
  "description": "Configure the SQL query and database connection for reading features from a database",
  "type": "object",
  "required": [
    "databaseUrl",
    "sql"
  ],
  "properties": {
    "databaseUrl": {
      "title": "Database URL",
      "description": "Database connection URL (e.g. `sqlite:///tests/sqlite/sqlite.db`, `mysql://user:password@localhost:3306/db`, `postgresql://user:password@localhost:5432/db`)",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
    },
    "sql": {
      "title": "SQL Query",
      "description": "SQL query expression to execute for retrieving data",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
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

## StatisticsCalculator
### Type
* processor
### Description
Calculates statistical aggregations on feature attributes with customizable expressions
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "StatisticsCalculator Parameters",
  "description": "Configuration for calculating statistical aggregations on feature attributes.",
  "type": "object",
  "required": [
    "calculations"
  ],
  "properties": {
    "calculations": {
      "title": "Calculations",
      "description": "List of statistical calculations to perform on grouped features",
      "type": "array",
      "items": {
        "$ref": "#/definitions/Calculation"
      }
    },
    "groupBy": {
      "title": "Group by",
      "description": "Attributes to group features by for aggregation. All of the inputs will be grouped if not specified.",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "groupId": {
      "title": "Group id",
      "description": "Optional attribute to store the group identifier. The ID will be formed by concatenating the values of the group_by attributes separated by '|'.",
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
    "Calculation": {
      "type": "object",
      "required": [
        "expr",
        "newAttribute"
      ],
      "properties": {
        "expr": {
          "title": "Calculation to perform",
          "allOf": [
            {
              "$ref": "#/definitions/Expr"
            }
          ]
        },
        "newAttribute": {
          "title": "New attribute name",
          "allOf": [
            {
              "$ref": "#/definitions/Attribute"
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
* complete
### Category
* Attribute

## SurfaceFootprintReplacer
### Type
* processor
### Description
Replace the geometry with its footprint
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "SurfaceFootprintReplacer Parameters",
  "description": "Configuration for replacing geometry with its footprint projection.",
  "type": "object",
  "properties": {
    "elevation": {
      "type": [
        "number",
        "null"
      ],
      "format": "double"
    },
    "lightDirection": {
      "type": [
        "array",
        "null"
      ],
      "items": {
        "type": "number",
        "format": "double"
      },
      "maxItems": 3,
      "minItems": 3
    },
    "shadowMode": {
      "type": [
        "string",
        "null"
      ]
    }
  }
}
```
### Input Ports
* default
### Output Ports
* footprint
* rejected
### Category
* Geometry

## ThreeDimensionBoxReplacer
### Type
* processor
### Description
Replace Geometry with 3D Box from Attributes
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "3D Box Replacer Parameters",
  "description": "Configure which attributes contain the minimum and maximum coordinates for creating a 3D box",
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
      "title": "Maximum X Attribute",
      "description": "Name of attribute containing the maximum X coordinate",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "maxY": {
      "title": "Maximum Y Attribute",
      "description": "Name of attribute containing the maximum Y coordinate",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "maxZ": {
      "title": "Maximum Z Attribute",
      "description": "Name of attribute containing the maximum Z coordinate",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "minX": {
      "title": "Minimum X Attribute",
      "description": "Name of attribute containing the minimum X coordinate",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "minY": {
      "title": "Minimum Y Attribute",
      "description": "Name of attribute containing the minimum Y coordinate",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "minZ": {
      "title": "Minimum Z Attribute",
      "description": "Name of attribute containing the minimum Z coordinate",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
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

## ThreeDimensionForcer
### Type
* processor
### Description
Convert 2D Geometry to 3D by Adding Z-Coordinates
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ThreeDimensionForcer Parameters",
  "description": "Configure how to convert 2D geometries to 3D by adding Z-coordinates",
  "type": "object",
  "properties": {
    "elevation": {
      "title": "Elevation",
      "description": "The Z-coordinate (elevation) value to add to all points. Can be a constant value or an expression. Defaults to 0.0 if not specified.",
      "anyOf": [
        {
          "$ref": "#/definitions/Expr"
        },
        {
          "type": "null"
        }
      ]
    },
    "preserveExistingZ": {
      "title": "Preserve Existing Z Values",
      "description": "If true, geometries that are already 3D will pass through unchanged. If false, existing Z values will be replaced with the specified elevation. Defaults to false.",
      "default": false,
      "type": "boolean"
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

## ThreeDimensionPlanarityRotator
### Type
* processor
### Description
Rotates a single or a set of 2D geometries in 3D space to align them horizontally.
### Parameters
* No parameters
### Input Ports
* default
### Output Ports
* default
* rejected
### Category
* Geometry

## ThreeDimensionRotator
### Type
* processor
### Description
Rotate 3D Geometry Around Arbitrary Axis
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "3D Rotator Parameters",
  "description": "Configure the 3D rotation parameters including axis, origin point, and angle",
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
      "title": "Angle in Degrees",
      "description": "Rotation angle in degrees around the specified axis",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
    },
    "directionX": {
      "title": "Direction X",
      "description": "X component of the rotation axis direction vector",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
    },
    "directionY": {
      "title": "Direction Y",
      "description": "Y component of the rotation axis direction vector",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
    },
    "directionZ": {
      "title": "Direction Z",
      "description": "Z component of the rotation axis direction vector",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
    },
    "originX": {
      "title": "Origin X",
      "description": "X coordinate of the rotation origin point",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
    },
    "originY": {
      "title": "Origin Y",
      "description": "Y coordinate of the rotation origin point",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
    },
    "originZ": {
      "title": "Origin Z",
      "description": "Z coordinate of the rotation origin point",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
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
Force 3D Geometry to 2D by Removing Z-Coordinates
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
Remove Redundant Vertices from Geometry
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
Reproject Vertical Coordinates Between Datums
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Vertical Reprojector Parameters",
  "description": "Configure the type of vertical datum conversion to apply",
  "type": "object",
  "required": [
    "reprojectorType"
  ],
  "properties": {
    "reprojectorType": {
      "title": "Reprojector Type",
      "description": "The type of vertical coordinate transformation to apply",
      "allOf": [
        {
          "$ref": "#/definitions/VerticalReprojectorType"
        }
      ]
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

## XMLFragmenter
### Type
* processor
### Description
Fragments large XML documents into smaller pieces based on specified element patterns
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "XMLFragmenter Parameters",
  "description": "Configuration for fragmenting XML documents into smaller pieces.",
  "oneOf": [
    {
      "description": "URL-based source configuration for XML fragmenting",
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
Validates XML documents against XSD schemas with success/failure routing
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

## XmlWriter
### Type
* sink
### Description
Writes features to XML files.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "XmlWriter Parameters",
  "description": "Configuration for writing features to XML files.",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "output": {
      "description": "Output path or expression for the XML file to create",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
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

## ZipFileWriter
### Type
* sink
### Description
Writes features to a zip file
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ZipFileWriter Parameters",
  "description": "Configuration for creating ZIP archive files from features.",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "output": {
      "description": "Output path",
      "allOf": [
        {
          "$ref": "#/definitions/Expr"
        }
      ]
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
