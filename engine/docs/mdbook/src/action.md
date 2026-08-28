# Actions

## Appearance Remover
### Type
* processor
### Description
Discards the materials, textures, and texture coordinates carried by a feature's geometry.
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* features
### Category
* Geometry

## Area Calculator
### Type
* processor
### Description
Calculates the true surface area of a feature's geometry and stores it in an attribute. A solid's area is the sum of its boundary surfaces, so a void's faces count toward it just like the exterior's.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Area Calculator Parameters",
  "description": "Where the measured surface area is stored on each feature.",
  "type": "object",
  "properties": {
    "outputAttribute": {
      "title": "Output Attribute",
      "description": "Attribute to store the measured surface area in. The attribute is always written, recording `0` when the geometry has no area or could not be measured, so a downstream step never has to handle a missing value.",
      "default": "area",
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
* features
### Output Ports
* features
### Category
* Geometry

## Area On Area Overlayer
### Type
* processor
### Description
Subdivides overlapping areas into non-overlapping pieces and records how many input features cover each piece. Inputs must be flat 2D geometries sharing one coordinate frame; place a Two Dimension Forcer or a Coordinate Frame Reprojector upstream to flatten or unify them.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Area On Area Overlayer Parameters",
  "description": "Sets which features are overlaid together, how small a piece of geometry has to be before it counts as noise, and what the resulting pieces record about the features they came from.",
  "type": "object",
  "properties": {
    "groupBy": {
      "title": "Group By Attributes",
      "description": "Attributes whose values decide which features are overlaid against each other — only features matching on all of them are compared. When omitted, every feature is overlaid against every other.",
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
      "description": "The size below which geometry is treated as noise rather than shape, in the unit of the input's coordinate frame: vertices closer together than this are merged before the overlay, so boundaries meant to coincide do, and an overlap covering less than its square is then discarded. Detail finer than the tolerance may therefore not survive the overlay intact. Defaults to zero, which merges and discards nothing.",
      "default": 0.0,
      "type": "number",
      "format": "double"
    },
    "attributeAccumulation": {
      "title": "Attribute Accumulation",
      "description": "Which attributes the resulting pieces keep.",
      "default": "useOneFeature",
      "allOf": [
        {
          "$ref": "#/definitions/AttributeAccumulation"
        }
      ]
    },
    "outputAttribute": {
      "title": "Overlap Count Attribute",
      "description": "Attribute that receives the number of input features covering the piece — two or more on `overlaps`, always one on `remnants`. Defaults to `overlayCount`.",
      "default": "overlayCount",
      "type": "string"
    },
    "listAttribute": {
      "title": "List Attribute",
      "description": "Attribute that receives one entry per covering feature, each holding that feature's own attributes. When omitted, no list is written.",
      "type": [
        "string",
        "null"
      ]
    }
  },
  "definitions": {
    "Attribute": {
      "type": "string"
    },
    "AttributeAccumulation": {
      "title": "Attribute Accumulation",
      "description": "Which attributes a resulting piece keeps.",
      "oneOf": [
        {
          "title": "Use Attributes From One Feature",
          "description": "Keeps the attributes of a single covering feature and discards the rest.",
          "type": "string",
          "enum": [
            "useOneFeature"
          ]
        },
        {
          "title": "Drop Incoming Attributes",
          "description": "Keeps no incoming attribute, so a piece carries only its overlap count and list attribute. The grouping attributes are dropped too.",
          "type": "string",
          "enum": [
            "dropAttributes"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* overlaps
* remnants
* rejected
### Category
* Geometry

## Attribute Aggregator
### Type
* processor
### Description
Groups features by attribute values and aggregates a computed value within each group, emitting one feature per group.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Attribute Aggregator Parameters",
  "description": "Configures how features are grouped and which value is aggregated within each group.",
  "type": "object",
  "required": [
    "aggregateAttributes",
    "calculationAttribute",
    "method"
  ],
  "properties": {
    "method": {
      "title": "Aggregation Method",
      "description": "How to aggregate the per-feature calculation value within each group: maximum, minimum, or count.",
      "allOf": [
        {
          "$ref": "#/definitions/Method"
        }
      ]
    },
    "aggregateAttributes": {
      "title": "Group-By Attributes",
      "description": "Attributes that define each group. Each entry reads a value from an existing attribute or an expression and writes it to a new attribute on the aggregated output feature.",
      "type": "array",
      "items": {
        "$ref": "#/definitions/AggregateAttribute"
      }
    },
    "calculationAttribute": {
      "title": "Result Attribute",
      "description": "Attribute on the aggregated feature that stores the computed result.",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "calculationValue": {
      "title": "Calculation Value",
      "description": "Constant integer used as the per-feature value. Takes precedence over the calculation expression when set.",
      "type": [
        "integer",
        "null"
      ],
      "format": "int64"
    },
    "calculation": {
      "title": "Calculation Expression",
      "description": "Expression evaluated to an integer per feature, used as the per-feature value when no calculation value is set.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  },
  "definitions": {
    "Method": {
      "oneOf": [
        {
          "title": "Maximum",
          "description": "Keeps the largest per-feature calculation value in the group.",
          "type": "string",
          "enum": [
            "max"
          ]
        },
        {
          "title": "Minimum",
          "description": "Keeps the smallest per-feature calculation value in the group.",
          "type": "string",
          "enum": [
            "min"
          ]
        },
        {
          "title": "Count",
          "description": "Sums the per-feature calculation value across the group; counts features when the value is 1.",
          "type": "string",
          "enum": [
            "count"
          ]
        }
      ]
    },
    "AggregateAttribute": {
      "type": "object",
      "required": [
        "newAttribute"
      ],
      "properties": {
        "newAttribute": {
          "title": "Output Attribute",
          "description": "Name of the attribute written to the aggregated feature that holds this group value.",
          "allOf": [
            {
              "$ref": "#/definitions/Attribute"
            }
          ]
        },
        "attribute": {
          "title": "Source Attribute",
          "description": "Existing attribute to read the group value from. Ignored when a group value expression is provided.",
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
          "title": "Group Value Expression",
          "description": "Expression that computes the group value. Takes precedence over the source attribute when both are set.",
          "type": [
            "object",
            "null"
          ],
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        }
      }
    },
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Attribute

## Attribute Bulk Array Joiner
### Type
* processor
### Description
Join Array Attributes Into Single Values
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Attribute Bulk Array Joiner Parameters",
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
* features
### Output Ports
* features
### Category
* Attribute

## Attribute Conversion Table
### Type
* processor
### Description
Transforms feature attributes by looking up values in a conversion table loaded from a file or provided inline (CSV, TSV, or JSON).
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Attribute Conversion Table Parameters",
  "description": "Configures the lookup table and the rules that map feature attribute values to replacement values.",
  "type": "object",
  "required": [
    "format",
    "rules"
  ],
  "properties": {
    "format": {
      "title": "Table Format",
      "description": "Format used to parse the conversion table.",
      "allOf": [
        {
          "$ref": "#/definitions/ConversionTableFormat"
        }
      ]
    },
    "rules": {
      "title": "Conversion Rules",
      "description": "Rules that match feature attribute values against table columns and write the looked-up result back to the feature.",
      "type": "array",
      "items": {
        "$ref": "#/definitions/AttributeConversionTableRule"
      }
    },
    "dataset": {
      "title": "Dataset URI",
      "description": "Path or URI of the conversion table file. Provide either this or inline data.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "inline": {
      "title": "Inline Table",
      "description": "Conversion table content provided directly as text. Used when no dataset URI is given.",
      "type": [
        "string",
        "null"
      ]
    }
  },
  "definitions": {
    "ConversionTableFormat": {
      "oneOf": [
        {
          "title": "CSV",
          "description": "Comma-separated values with a header row.",
          "type": "string",
          "enum": [
            "csv"
          ]
        },
        {
          "title": "TSV",
          "description": "Tab-separated values with a header row.",
          "type": "string",
          "enum": [
            "tsv"
          ]
        },
        {
          "title": "JSON",
          "description": "JSON array of objects, or a single object, where each object is a table row.",
          "type": "string",
          "enum": [
            "json"
          ]
        }
      ]
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
        "featureFroms": {
          "title": "Source Attributes",
          "description": "Feature attributes whose values form the key looked up in the table.",
          "type": "array",
          "items": {
            "$ref": "#/definitions/Attribute"
          }
        },
        "featureTo": {
          "title": "Target Attribute",
          "description": "Feature attribute that receives the looked-up value.",
          "allOf": [
            {
              "$ref": "#/definitions/Attribute"
            }
          ]
        },
        "conversionTableKeys": {
          "title": "Lookup Key Columns",
          "description": "Table columns matched against the source attribute values.",
          "type": "array",
          "items": {
            "type": "string"
          }
        },
        "conversionTableTo": {
          "title": "Result Column",
          "description": "Table column whose value is written to the target attribute.",
          "type": "string"
        }
      }
    },
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Attribute

## Attribute Duplicate Filter
### Type
* processor
### Description
Remove Duplicate Features Based on Attribute Values
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Attribute Duplicate Filter Parameters",
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
* features
### Output Ports
* features
### Category
* Attribute

## Attribute File Path Info Extractor
### Type
* processor
### Description
Extract File System Information from Path Attributes
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Attribute File Path Info Extractor Parameters",
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
* features
### Output Ports
* features
* rejected
### Category
* Attribute

## Attribute Flattener
### Type
* processor
### Description
Flattens map-valued attributes into individual top-level attributes, replacing each map with its key-value entries.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Attribute Flattener Parameters",
  "description": "Configures which map-valued attributes are expanded into top-level attributes.",
  "type": "object",
  "required": [
    "attributes"
  ],
  "properties": {
    "attributes": {
      "title": "Attributes to Flatten",
      "description": "Map-valued attributes to expand; each nested key becomes a top-level attribute and the original map is removed. Non-map attributes are left unchanged.",
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
* features
### Output Ports
* features
### Category
* Attribute

## Attribute Manager
### Type
* processor
### Description
Creates, converts, renames, or removes feature attributes based on a configurable list of operations.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Attribute Manager Parameters",
  "description": "Defines the ordered list of operations that create, convert, rename, or remove feature attributes.",
  "type": "object",
  "required": [
    "operations"
  ],
  "properties": {
    "operations": {
      "title": "Attribute Operations",
      "description": "Operations applied to each feature in order. Each entry names the target attribute, the method to apply, and an optional value expression.",
      "type": "array",
      "items": {
        "$ref": "#/definitions/Operation"
      }
    }
  },
  "definitions": {
    "Operation": {
      "type": "object",
      "required": [
        "attribute",
        "method"
      ],
      "properties": {
        "attribute": {
          "title": "Attribute Name",
          "description": "Name of the attribute to create, convert, rename, or remove.",
          "type": "string"
        },
        "method": {
          "title": "Method",
          "description": "Operation to apply to the attribute.",
          "allOf": [
            {
              "$ref": "#/definitions/Method"
            }
          ]
        },
        "value": {
          "title": "Value",
          "description": "Expression evaluated against the feature. Supplies the new value for Create and Convert, or the new attribute name for Rename. Ignored for Remove.",
          "type": [
            "object",
            "null"
          ],
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr",
                "string"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        }
      }
    },
    "Method": {
      "oneOf": [
        {
          "title": "Convert",
          "description": "Replaces the value of an existing attribute with the result of the value expression. Features that do not already have the attribute are left unchanged.",
          "type": "string",
          "enum": [
            "convert"
          ]
        },
        {
          "title": "Create",
          "description": "Sets the attribute to the result of the value expression, creating it or overwriting any existing value.",
          "type": "string",
          "enum": [
            "create"
          ]
        },
        {
          "title": "Rename",
          "description": "Renames the attribute to the name produced by the value expression. Skipped when the attribute is absent or a target with that name already exists.",
          "type": "string",
          "enum": [
            "rename"
          ]
        },
        {
          "title": "Remove",
          "description": "Removes the attribute from the feature.",
          "type": "string",
          "enum": [
            "remove"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Attribute

## Attribute Mapper
### Type
* processor
### Description
Replaces a feature's attributes with a new set built from mapping rules, each deriving its value from an expression, another attribute, or a nested map entry.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Attribute Mapper Parameters",
  "description": "Configures the mapping rules that build the output feature's attributes.",
  "type": "object",
  "required": [
    "mappers"
  ],
  "properties": {
    "mappers": {
      "title": "Mapping Rules",
      "description": "Ordered list of rules; each produces one or more output attributes. The output feature contains only the attributes produced here.",
      "type": "array",
      "items": {
        "$ref": "#/definitions/Mapper"
      }
    }
  },
  "definitions": {
    "Mapper": {
      "type": "object",
      "properties": {
        "attribute": {
          "title": "Target Attribute",
          "description": "Name of the attribute to set, taking its value from the expression, source attribute, or parent/child pair. Leave empty to use the multiple-values expression instead.",
          "type": [
            "string",
            "null"
          ]
        },
        "expr": {
          "title": "Value Expression",
          "description": "Expression evaluated to produce the attribute value. Evaluation errors yield a null value.",
          "type": [
            "object",
            "null"
          ],
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr",
                "string"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "valueAttribute": {
          "title": "Source Attribute",
          "description": "Existing attribute to copy the value from. Yields null when the attribute is absent.",
          "type": [
            "string",
            "null"
          ]
        },
        "parentAttribute": {
          "title": "Parent Attribute",
          "description": "Map-valued attribute containing the value to copy. Used together with the child attribute.",
          "type": [
            "string",
            "null"
          ]
        },
        "childAttribute": {
          "title": "Child Attribute",
          "description": "Key within the parent map whose value is copied to the target attribute.",
          "type": [
            "string",
            "null"
          ]
        },
        "multipleExpr": {
          "title": "Multiple-Values Expression",
          "description": "Expression returning a map whose entries are added as attributes. Used when no target attribute is set.",
          "type": [
            "object",
            "null"
          ],
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Attribute

## Attribute Range Mapper
### Type
* processor
### Description
Classifies a numeric attribute by looking its value up in a table of ranges and writing the matched range's output value to another attribute.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Attribute Range Mapper Parameters",
  "description": "Defines the attribute to classify, the table of ranges to match it against, and where the matched value is written.",
  "type": "object",
  "required": [
    "inputAttribute",
    "outputAttribute",
    "rangeTable"
  ],
  "properties": {
    "inputAttribute": {
      "title": "Input Attribute",
      "description": "Attribute holding the value to classify. Numbers are used directly, numeric strings are parsed, and booleans count as 1 and 0. Any other type is treated as unclassifiable and takes the default value.",
      "type": "string"
    },
    "outputAttribute": {
      "title": "Output Attribute",
      "description": "Attribute the matched value is written to. An existing value is overwritten.",
      "type": "string"
    },
    "rangeTable": {
      "title": "Range Lookup Table",
      "description": "Ranges to test the input against, in order. The first match wins, so overlapping ranges resolve to whichever is listed first.",
      "type": "array",
      "items": {
        "$ref": "#/definitions/RangeEntry"
      }
    },
    "defaultValue": {
      "title": "Default Value",
      "description": "Value written when no range matches, and also when the input attribute is absent or is not a number, numeric string, or boolean. When omitted, those features pass through with the output attribute left unset rather than being rejected.",
      "anyOf": [
        {
          "$ref": "#/definitions/MappedValue"
        },
        {
          "type": "null"
        }
      ]
    }
  },
  "definitions": {
    "RangeEntry": {
      "title": "Range Entry",
      "type": "object",
      "required": [
        "from",
        "outputValue",
        "to"
      ],
      "properties": {
        "from": {
          "title": "From (Minimum)",
          "description": "Lower bound of the range, inclusive.",
          "type": "number",
          "format": "double"
        },
        "to": {
          "title": "To (Maximum)",
          "description": "Upper bound of the range, exclusive — a value equal to it falls into the next range. Setting it equal to the lower bound makes the entry match that one exact value instead.",
          "type": "number",
          "format": "double"
        },
        "outputValue": {
          "title": "Output Value",
          "description": "Value written to the output attribute when the input falls in this range.",
          "allOf": [
            {
              "$ref": "#/definitions/MappedValue"
            }
          ]
        }
      }
    },
    "MappedValue": {
      "title": "Mapped Value",
      "description": "A value written to an attribute. Accepts text, a number, or true/false, written as the type given — `\"3\"` stays text and `3` stays a number.",
      "anyOf": [
        {
          "title": "Text",
          "description": "Written as text.",
          "type": "string"
        },
        {
          "title": "Number",
          "description": "Written as a number.",
          "type": "number"
        },
        {
          "title": "True or False",
          "description": "Written as a true/false value.",
          "type": "boolean"
        }
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Attribute

## Attribute Table Extractor
### Type
* processor
### Description
Moves values between nested map/list attribute paths, following a table of source/destination path pairs keyed by a feature type attribute. A destination path with more than one segment creates nested maps as needed.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Attribute Table Extractor Parameters",
  "description": "Configures the table of source/destination path pairs used to move nested attribute values. Supply the table either as a file, via Dataset URI, or directly, via Inline Table — one of the two is required.",
  "type": "object",
  "properties": {
    "dataset": {
      "title": "Dataset URI",
      "description": "Path or URI of a JSON file holding the extraction table. Provide either this or Inline Table.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "inline": {
      "title": "Inline Table",
      "description": "Extraction table given directly, keyed by feature type. Each key is a feature type name as it appears in the type attribute, and its value is the list of rules applied to features of that type. Provide either this or Dataset URI.",
      "type": [
        "object",
        "null"
      ],
      "additionalProperties": {
        "type": "array",
        "items": {
          "$ref": "#/definitions/ExtractRule"
        }
      }
    },
    "typeAttribute": {
      "title": "Feature Type Attribute",
      "description": "Attribute whose value selects which rule set in the table applies to the feature. Defaults to `__citygml_feature_type`.",
      "type": [
        "string",
        "null"
      ]
    }
  },
  "definitions": {
    "ExtractRule": {
      "title": "Extraction Rule",
      "description": "Moves one value from a source path to a destination path.\n\nBoth paths are chains of attribute keys separated by **spaces**, not dots — the keys themselves are qualified XML names such as `bldg:measuredHeight`, which may contain colons and dots but never whitespace.",
      "type": "object",
      "required": [
        "destinationPath",
        "sourcePath"
      ],
      "properties": {
        "destinationPath": {
          "title": "Destination Path",
          "description": "Where the value is written, as a space-separated chain of keys. A single segment writes a top-level attribute; several segments write into a nested map, creating it and any missing intermediate map as needed.",
          "type": "string"
        },
        "sourcePath": {
          "title": "Source Path",
          "description": "Where the value is read from, as a space-separated chain of keys walked from the feature's top level. A segment matches either a key in a map or the first matching element of a list, so it works whether a wrapper element appears once or many times.",
          "type": "string"
        },
        "dataType": {
          "title": "Value Type",
          "description": "Converts the extracted value before writing it. Leave unset to write it unchanged.",
          "default": null,
          "anyOf": [
            {
              "$ref": "#/definitions/ExtractDataType"
            },
            {
              "type": "null"
            }
          ]
        }
      }
    },
    "ExtractDataType": {
      "oneOf": [
        {
          "title": "Integer",
          "description": "Parses a string value as an integer.",
          "type": "string",
          "enum": [
            "int"
          ]
        },
        {
          "title": "Float",
          "description": "Parses a string value as a floating point number.",
          "type": "string",
          "enum": [
            "float"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Attribute

## Boundary Extractor
### Type
* processor
### Description
Replaces a geometry with its boundary: the endpoints of a curve, the boundary rings of a surface, and the bounding shells of a volume.
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* boundary
* no-boundary
* rejected
### Category
* Geometry

## Bounds Extractor
### Type
* processor
### Description
Extracts the bounding box coordinates of a feature's geometry and stores them as named attributes.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Bounds Extractor Parameters",
  "description": "Configure the attribute names each bound is stored under. Every parameter is optional; a bound left unset is stored under its default name. The z bounds are written only when the geometry has an elevation.",
  "type": "object",
  "properties": {
    "xmin": {
      "title": "Minimum X Attribute",
      "description": "Attribute to store the minimum X coordinate in. Defaults to `xmin`.",
      "anyOf": [
        {
          "$ref": "#/definitions/Attribute"
        },
        {
          "type": "null"
        }
      ]
    },
    "xmax": {
      "title": "Maximum X Attribute",
      "description": "Attribute to store the maximum X coordinate in. Defaults to `xmax`.",
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
      "description": "Attribute to store the minimum Y coordinate in. Defaults to `ymin`.",
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
      "description": "Attribute to store the maximum Y coordinate in. Defaults to `ymax`.",
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
      "description": "Attribute to store the minimum Z coordinate in. Defaults to `zmin`.",
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
      "description": "Attribute to store the maximum Z coordinate in. Defaults to `zmax`.",
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
* features
### Output Ports
* features
* rejected
### Category
* Geometry

## Bufferer
### Type
* processor
### Description
Creates a buffer polygon around each input geometry at a specified distance.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Bufferer Parameters",
  "description": "Configure the shape and extent of the buffer created around each geometry.",
  "type": "object",
  "required": [
    "bufferType",
    "distance"
  ],
  "properties": {
    "bufferType": {
      "title": "Buffer Type",
      "description": "Shape of buffer to create around the input geometry.",
      "allOf": [
        {
          "$ref": "#/definitions/BufferType"
        }
      ]
    },
    "distance": {
      "title": "Distance",
      "description": "How far the buffer extends from the original geometry, in the units of the geometry's coordinate system. A negative distance contracts it.",
      "type": "number",
      "format": "double"
    },
    "interpolationAngle": {
      "title": "Interpolation Angle",
      "description": "Angular step in degrees used to approximate the rounded caps, joins, and discs of the buffer outline. A smaller angle produces a smoother outline. Values outside the range of 1.8 to 45 degrees are clamped to it. Defaults to 11.25 degrees when omitted.",
      "type": [
        "number",
        "null"
      ],
      "format": "double"
    }
  },
  "definitions": {
    "BufferType": {
      "oneOf": [
        {
          "title": "2D Area Buffer",
          "description": "Creates an areal buffer around the input geometry: a 2D geometry is buffered in its coordinate plane, a planar 3D polygon within its own plane. An elevation shared by the buffered geometry is kept.",
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
* features
### Output Ports
* features
* rejected
### Category
* Geometry

## Bulk Attribute Renamer
### Type
* processor
### Description
Renames feature attributes in bulk by adding or removing a prefix or suffix, or replacing text.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Bulk Attribute Renamer Parameters",
  "description": "Selects which attributes to rename and how their names are transformed.",
  "type": "object",
  "required": [
    "renameAction",
    "renameType",
    "renameValue"
  ],
  "properties": {
    "renameType": {
      "title": "Rename Scope",
      "description": "Scope of the rename operation: all attributes or a selected subset.",
      "allOf": [
        {
          "$ref": "#/definitions/RenameType"
        }
      ]
    },
    "renameAction": {
      "title": "Rename Operation",
      "description": "Transformation applied to each attribute name.",
      "allOf": [
        {
          "$ref": "#/definitions/RenameAction"
        }
      ]
    },
    "renameValue": {
      "title": "Text Value",
      "description": "Text to add as a prefix or suffix, remove, or use as the replacement, depending on the selected operation.",
      "type": "string"
    },
    "textToFind": {
      "title": "Text Pattern to Find",
      "description": "Regular expression matched against attribute names. Required for the replaceText operation.",
      "type": [
        "string",
        "null"
      ]
    },
    "selectedAttributes": {
      "title": "Selected Attribute Names",
      "description": "Attribute names to rename. Required when the rename scope is set to a selected subset.",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "type": "string"
      }
    }
  },
  "definitions": {
    "RenameType": {
      "oneOf": [
        {
          "title": "All Attributes",
          "description": "Renames every attribute on the feature.",
          "type": "string",
          "enum": [
            "all"
          ]
        },
        {
          "title": "Selected Attributes",
          "description": "Renames only the attributes listed in selectedAttributes.",
          "type": "string",
          "enum": [
            "selected"
          ]
        }
      ]
    },
    "RenameAction": {
      "oneOf": [
        {
          "title": "Add Prefix",
          "description": "Prepends the text value to each attribute name.",
          "type": "string",
          "enum": [
            "addPrefix"
          ]
        },
        {
          "title": "Add Suffix",
          "description": "Appends the text value to each attribute name.",
          "type": "string",
          "enum": [
            "addSuffix"
          ]
        },
        {
          "title": "Remove Prefix",
          "description": "Removes the text value from the start of each attribute name.",
          "type": "string",
          "enum": [
            "removePrefix"
          ]
        },
        {
          "title": "Remove Suffix",
          "description": "Removes the text value from the end of each attribute name.",
          "type": "string",
          "enum": [
            "removeSuffix"
          ]
        },
        {
          "title": "Replace Text",
          "description": "Replaces text matched by the find pattern with the text value, using regular expressions.",
          "type": "string",
          "enum": [
            "replaceText"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Attribute

## CSG Builder
### Type
* processor
### Description
Pairs each left solid with the right solid that shares its pair value and emits the union, the intersection and the difference of the pair as unevaluated Constructive Solid Geometry trees. The trees describe the boolean without computing it, so a CSG Evaluator downstream turns the branch you keep into a solid.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CSG Builder Parameters",
  "description": "Sets how the two input streams are paired up and what the resulting trees record about the solids they were built from.",
  "type": "object",
  "required": [
    "pairId"
  ],
  "properties": {
    "pairId": {
      "title": "Pair ID",
      "description": "Expression evaluated on every feature to produce the value that pairs it up: a left feature and a right feature that evaluate to the same value are combined. A feature whose partner never arrives is rejected.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "listAttribute": {
      "title": "List Attribute",
      "description": "Attribute that receives one entry for the left solid and one for the right, each holding that feature's own attributes. When omitted, no list is written and the resulting trees carry no attributes at all.",
      "type": [
        "string",
        "null"
      ]
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

## CSG Evaluator
### Type
* processor
### Description
Computes the solid a Constructive Solid Geometry tree describes, replacing the tree with the result. Operands must be closed, outward-wound solids in a projected coordinate reference, since the vertex tolerance is a distance; a tree whose result encloses no volume leaves on the empty port.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CSG Evaluator Parameters",
  "description": "Sets how closely the operands' vertices must line up for the boolean to treat them as one point.",
  "type": "object",
  "properties": {
    "tolerance": {
      "title": "Tolerance",
      "description": "Distance below which a vertex counts as lying on a cutting plane and two vertices count as one, in the unit of the operands' coordinate reference. Defaults to a distance small enough that only near-identical vertices merge.",
      "default": 1e-9,
      "type": "number",
      "format": "double"
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* empty
* rejected
### Category
* Geometry

## CSV Reader
### Type
* source
### Description
Reads features from CSV and TSV files.
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
    "format": {
      "title": "File Format",
      "description": "Delimiter format of the input file.",
      "allOf": [
        {
          "$ref": "#/definitions/CsvFormat"
        }
      ]
    },
    "encoding": {
      "title": "Character Encoding",
      "description": "Character encoding of the input file, such as \"UTF-8\" or \"Shift-JIS\". Defaults to UTF-8 when omitted; labels are case-insensitive.",
      "type": [
        "string",
        "null"
      ]
    },
    "dataset": {
      "title": "File Path",
      "description": "Expression that returns the path to the input file, either a literal path or a variable reference.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
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
    },
    "headerRows": {
      "title": "Header Row Count",
      "description": "Number of consecutive rows that make up the header (default: 1). When 0, column names are auto-generated as \"column1\", \"column2\", and so on; when greater than 1, names are formed by joining values from each header row with \"_\".",
      "type": [
        "integer",
        "null"
      ],
      "format": "uint",
      "minimum": 0.0
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
            "column"
          ],
          "properties": {
            "column": {
              "title": "WKT Column Name",
              "description": "Name of the column containing WKT geometry",
              "type": "string"
            }
          }
        },
        {
          "title": "Coordinate Columns",
          "description": "Geometry stored as separate X, Y, (optional Z) columns",
          "type": "object",
          "required": [
            "xColumn",
            "yColumn"
          ],
          "properties": {
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
* features
### Category
* Input

## CSV Writer
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
    "output": {
      "title": "Output File",
      "description": "Output path or expression for the CSV/TSV file to create.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "format": {
      "title": "File Format",
      "description": "File format to write: CSV (comma-separated) or TSV (tab-separated).",
      "allOf": [
        {
          "$ref": "#/definitions/CsvFormat"
        }
      ]
    },
    "geometry": {
      "title": "Geometry Configuration",
      "description": "Optional configuration for exporting geometry to CSV columns.",
      "anyOf": [
        {
          "$ref": "#/definitions/GeometryExportConfig"
        },
        {
          "type": "null"
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
    "GeometryExportConfig": {
      "title": "Geometry Export Configuration",
      "description": "Configure how geometry data is written to CSV columns",
      "type": "object",
      "oneOf": [
        {
          "title": "WKT Column",
          "description": "Write geometry as Well-Known Text in a single column",
          "type": "object",
          "required": [
            "column"
          ],
          "properties": {
            "column": {
              "title": "WKT Column Name",
              "description": "Name of the column to write WKT geometry",
              "type": "string"
            }
          }
        },
        {
          "title": "Coordinate Columns",
          "description": "Write geometry as separate X, Y, (optional Z) columns\nNote: Only supports Point geometries. Non-point geometries will be skipped with a warning.",
          "type": "object",
          "required": [
            "xColumn",
            "yColumn"
          ],
          "properties": {
            "xColumn": {
              "title": "X Column Name",
              "description": "Name of the column for X coordinate (longitude)",
              "type": "string"
            },
            "yColumn": {
              "title": "Y Column Name",
              "description": "Name of the column for Y coordinate (latitude)",
              "type": "string"
            },
            "zColumn": {
              "title": "Z Column Name",
              "description": "Optional name of the column for Z coordinate (elevation)",
              "type": [
                "string",
                "null"
              ]
            }
          }
        }
      ],
      "properties": {
        "epsgColumn": {
          "title": "EPSG Column Name",
          "description": "Optional name of a column to write the geometry's EPSG code into. Left empty when the geometry does not resolve to a single EPSG code (no coordinate reference system, or more than one).",
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
* features
### Output Ports
### Category
* Output

## CZML Reader
### Type
* source
### Description
Reads geographic features from CZML (Cesium Language) files for 3D visualization, with support for time-dynamic properties and timeseries data
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CZML Reader Parameters",
  "description": "Configuration for reading CZML files as geographic features.",
  "type": "object",
  "properties": {
    "force2d": {
      "title": "Force 2D",
      "description": "If true, forces all geometries to be 2D (ignoring Z values)",
      "default": false,
      "type": "boolean"
    },
    "skipDocumentPacket": {
      "title": "Skip Document Packet",
      "description": "If true, skips the document packet (first packet with version/clock info)",
      "default": true,
      "type": "boolean"
    },
    "timeSampling": {
      "title": "Time Sampling Strategy",
      "description": "How to handle time-dynamic properties in CZML packets. Defaults to \"preserveRaw\" for lossless round-trip with CZML Writer.",
      "default": "preserveRaw",
      "allOf": [
        {
          "$ref": "#/definitions/TimeSamplingStrategy"
        }
      ]
    },
    "dataset": {
      "title": "File Path",
      "description": "Expression that returns the path to the input file, either a literal path or a variable reference.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  },
  "definitions": {
    "TimeSamplingStrategy": {
      "description": "Strategy for handling time-dynamic CZML properties.",
      "oneOf": [
        {
          "description": "Extract all time-tagged samples as separate features, each with a `czml.timestamp` and `czml.timeOffset` attribute. Useful when you need per-sample processing in downstream actions.",
          "type": "string",
          "enum": [
            "allSamples"
          ]
        },
        {
          "description": "Keep the first sample only (static geometry). Use this for workflows that don't need timeseries data.",
          "type": "string",
          "enum": [
            "firstSampleOnly"
          ]
        },
        {
          "description": "Embed the full timeseries in one feature per entity. The feature geometry uses the first sample, `czml.timeseries` holds all position samples as a JSON array, and all other CZML packet properties (point, path, orientation, ellipsoid, etc.) are preserved as `czml.<key>` attributes for faithful round-trip through CZML Writer.",
          "type": "string",
          "enum": [
            "preserveRaw"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
### Output Ports
* features
### Category
* File

## CZML Writer
### Type
* sink
### Description
Export features as CZML for Cesium visualization. Supports static entities and time-animated timeseries. Configure timeField, groupTimeseriesBy, and epoch (for numeric times) to enable animation.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CZML Writer Parameters",
  "description": "Configuration for writing geographic features to CZML files. Supports both static entities and time-dynamic entities with interpolated position samples.\n\n## Timeseries Configuration\n\nTo create time-animated entities in Cesium, configure at least the first two parameters below; configure `epoch` only when using numeric time offsets: 1. `timeField` - Attribute containing time values 2. `groupTimeseriesBy` - Attribute to group features into entities 3. `epoch` (optional) - Base time used when `timeField` contains numeric offsets\n\n### Example with ISO 8601 timestamps: ```yaml - action: CZML Writer with: output: \"vehicles.czml\" timeField: \"timestamp\"           # Contains \"2024-01-01T00:00:00Z\", etc. groupTimeseriesBy: \"vehicleId\"   # Groups by vehicle ID interpolationAlgorithm: \"LAGRANGE\" interpolationDegree: 5 ```\n\n### Example with numeric time offsets: ```yaml - action: CZML Writer with: output: \"sensors.czml\" timeField: \"timeOffset\"          # Contains numeric values: 0, 60, 120, etc. groupTimeseriesBy: \"sensorId\" epoch: \"2024-01-01T00:00:00Z\"    # Base time for offsets interpolationAlgorithm: \"LINEAR\" ```",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "output": {
      "title": "Output File Path",
      "description": "Path where the CZML file will be written",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
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
    "timeField": {
      "title": "Time Field",
      "description": "Attribute containing the timestamp for each feature. Supports two formats: - **ISO 8601 strings**: e.g., \"2024-01-01T00:00:00Z\", \"2024-01-01T12:30:45+09:00\" - **Numeric values**: Seconds as offset from epoch (e.g., \"0\", \"60\", \"120.5\")\n\nWhen set together with `groupTimeseriesBy`, features sharing the same group key are combined into a single CZML entity with time-tagged position samples for animation in Cesium.\n\n**Example workflow configuration:** ```yaml - action: CZML Writer with: output: \"output.czml\" timeField: \"timestamp\" groupTimeseriesBy: \"vehicleId\" epoch: \"2024-01-01T00:00:00Z\"  # Optional for numeric times (auto-defaults to Unix epoch) ```",
      "anyOf": [
        {
          "$ref": "#/definitions/Attribute"
        },
        {
          "type": "null"
        }
      ]
    },
    "epoch": {
      "title": "Epoch",
      "description": "Reference time (ISO 8601 format) used as the base for numeric time offsets.\n\n**When to use:** - Optional but recommended when `timeField` contains numeric values (e.g., \"0\", \"60\", \"3600\") - Not needed when `timeField` contains ISO 8601 datetime strings\n\n**Format:** ISO 8601 datetime string with timezone - Examples: \"2024-01-01T00:00:00Z\", \"2024-06-15T09:00:00+09:00\"\n\n**Auto-detection:** If omitted and all time values are numeric, automatically defaults to Unix epoch \"1970-01-01T00:00:00Z\". For custom time ranges, explicitly set this parameter to your desired base time.\n\n**Example:** ```yaml epoch: \"2024-01-01T00:00:00Z\"  # Time value \"60\" means 2024-01-01T00:01:00Z ```",
      "type": [
        "string",
        "null"
      ]
    },
    "interpolationAlgorithm": {
      "title": "Interpolation Algorithm",
      "description": "Algorithm used by Cesium to interpolate between time-tagged samples.",
      "default": "LINEAR",
      "allOf": [
        {
          "$ref": "#/definitions/InterpolationAlgorithm"
        }
      ]
    },
    "interpolationDegree": {
      "title": "Interpolation Degree",
      "description": "Degree of interpolation (1 for LINEAR, 5 typical for LAGRANGE).",
      "default": 1,
      "type": "integer",
      "format": "uint32",
      "minimum": 0.0
    },
    "groupTimeseriesBy": {
      "title": "Group Timeseries By",
      "description": "Attribute used to group features into a single time-dynamic CZML entity. Features with the same value for this attribute are merged into one packet with time-tagged positions.",
      "anyOf": [
        {
          "$ref": "#/definitions/Attribute"
        },
        {
          "type": "null"
        }
      ]
    },
    "colorAttribute": {
      "title": "Color Attribute",
      "description": "Attribute containing a hex color string (e.g., \"#ffd8c0\") for polygon fill. Used when polygon geometry is auto-converted from the feature geometry.",
      "anyOf": [
        {
          "$ref": "#/definitions/Attribute"
        },
        {
          "type": "null"
        }
      ]
    },
    "opacity": {
      "title": "Opacity",
      "description": "Alpha value (0–255) for polygon fill color. Default: 180.",
      "default": 180,
      "type": "integer",
      "format": "uint8",
      "minimum": 0.0
    },
    "heightAttribute": {
      "title": "Height Attribute",
      "description": "Attribute containing a numeric value for polygon extrusion height. When set, polygons are extruded from ground to this height value.",
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
    "InterpolationAlgorithm": {
      "description": "Interpolation algorithm for Cesium time-dynamic properties.",
      "oneOf": [
        {
          "description": "Linear interpolation between samples (degree 1).",
          "type": "string",
          "enum": [
            "LINEAR"
          ]
        },
        {
          "description": "Lagrange polynomial interpolation for smooth curves (typical degree 5).",
          "type": "string",
          "enum": [
            "LAGRANGE"
          ]
        },
        {
          "description": "Hermite spline interpolation using tangent data.",
          "type": "string",
          "enum": [
            "HERMITE"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
### Category
* File

## Center Point Replacer
### Type
* processor
### Description
Replace Feature Geometry with Center Point
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CenterPointReplacerParam",
  "type": "object",
  "properties": {
    "mode": {
      "description": "The method used to compute the replacement center point.",
      "default": "centerOfGravity",
      "allOf": [
        {
          "$ref": "#/definitions/CenterPointMode"
        }
      ]
    }
  },
  "definitions": {
    "CenterPointMode": {
      "description": "Method used to compute the center point of a geometry.",
      "oneOf": [
        {
          "description": "Computes the centroid (center of gravity) of the geometry.",
          "type": "string",
          "enum": [
            "centerOfGravity"
          ]
        },
        {
          "description": "Computes the center of the geometry's bounding box.",
          "type": "string",
          "enum": [
            "boundingBoxCenter"
          ]
        },
        {
          "description": "Computes a point guaranteed to lie inside the geometry (pole of inaccessibility).",
          "type": "string",
          "enum": [
            "anyInsidePoint"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* point
* rejected
### Category
* Geometry

## Cesium 3D Tiles Writer
### Type
* sink
### Description
Writes features to Cesium 3D Tiles format for 3D web visualization.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Cesium3DTilesWriter Parameters",
  "description": "Configuration for writing features to Cesium 3D Tiles.",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "output": {
      "title": "Output Path",
      "description": "Directory path where the 3D Tiles will be written.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "targetTileSize": {
      "title": "Target Tile Size",
      "description": "Target content size per tile, in bytes. Tiles are split when they'd exceed it and merged with neighbours when they'd otherwise be smaller; a single feature that alone exceeds it is kept whole (features are never clipped). A value of 0 disables merging and splits every feature into its own content. Defaults to 1,048,576 (1 MiB).",
      "type": [
        "integer",
        "null"
      ],
      "format": "uint64",
      "minimum": 0.0
    },
    "dracoCompression": {
      "title": "Draco Compression",
      "description": "Whether to compress mesh geometry with Draco. Defaults to true.",
      "default": true,
      "type": "boolean"
    },
    "computeFlatNormal": {
      "title": "Compute Flat Normals",
      "description": "Compute per-polygon flat normals for lighting. Defaults to true. When disabled, no normals are written and the mesh is smaller, but the tile carries no lighting data (a viewer must derive flat normals itself).",
      "default": true,
      "type": "boolean"
    },
    "texelSize": {
      "title": "Texel Size",
      "description": "Target texel size in metres per pixel. Textures finer than this are downsampled to it. Defaults to 0, which keeps full texture detail.",
      "type": [
        "number",
        "null"
      ],
      "format": "double"
    },
    "atlasSize": {
      "title": "Atlas Size",
      "description": "Maximum texture atlas dimension in pixels. Textures exceeding this spill onto additional atlas pages; a single texture larger than it is downsampled to fit. Defaults to 2048.",
      "type": [
        "integer",
        "null"
      ],
      "format": "uint32",
      "maximum": 65536.0,
      "minimum": 1.0
    },
    "atlasExtrusion": {
      "title": "Atlas Extrusion",
      "description": "Ring of pixels blitted around each texture region in the atlas to stop bilinear bleed between neighbouring regions. Defaults to 0 (disabled).",
      "type": [
        "integer",
        "null"
      ],
      "format": "uint32",
      "maximum": 65536.0,
      "minimum": 0.0
    },
    "textureCodec": {
      "title": "Texture Codec",
      "description": "Image codec for atlas pages. Defaults to `KTX2/ETC1S`; select `Untextured` to attach no textures.",
      "default": "KTX2/ETC1S",
      "allOf": [
        {
          "$ref": "#/definitions/TextureCodec"
        }
      ]
    },
    "schemaKey": {
      "title": "Schema Key",
      "description": "Attribute key whose value identifies the schema type and determines the output filename: all features sharing the same value are written to the same file. This attribute is excluded from output.",
      "type": [
        "string",
        "null"
      ]
    },
    "skipUnexposedAttributes": {
      "title": "Skip Unexposed Attributes",
      "description": "Whether to skip attributes whose keys begin with a double underscore.",
      "type": [
        "boolean",
        "null"
      ]
    },
    "arrayMapSeparator": {
      "title": "Array/Map Separator",
      "description": "Separator joining a nested array or map attribute to its child key or index when flattening it into metadata columns. Leave unset to drop array and map attributes from the output entirely.",
      "type": [
        "string",
        "null"
      ]
    }
  },
  "definitions": {
    "TextureCodec": {
      "title": "Texture Codec",
      "description": "Texture image codec for the new-geometry writer's atlas pages.",
      "oneOf": [
        {
          "description": "KTX2 with Basis Universal UASTC supercompression (`KHR_texture_basisu`): higher quality, larger files.",
          "type": "string",
          "enum": [
            "KTX2/UASTC"
          ]
        },
        {
          "description": "KTX2 with Basis Universal ETC1S supercompression (`KHR_texture_basisu`): smaller files, lower quality.",
          "type": "string",
          "enum": [
            "KTX2/ETC1S"
          ]
        },
        {
          "description": "PNG, lossless with alpha.",
          "type": "string",
          "enum": [
            "PNG"
          ]
        },
        {
          "description": "JPEG, lossy and opaque (alpha is dropped).",
          "type": "string",
          "enum": [
            "JPEG"
          ]
        },
        {
          "description": "Attach no textures; textured geometry falls back to its neutral colour.",
          "type": "string",
          "enum": [
            "Untextured"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* features
* schema
### Output Ports
### Category
* Output

## CityGML Reader
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
      "description": "Expression that returns the path to the input file, either a literal path or a variable reference.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "flatten": {
      "title": "Flatten Feature Tree",
      "description": "When enabled, extracts nested child city objects as separate features, each tagged with `parentId` and `parentType` attributes. Defaults to false.",
      "type": [
        "boolean",
        "null"
      ]
    }
  }
}
```
### Input Ports
### Output Ports
* features
### Category
* Input

## CityGML Writer
### Type
* sink
### Description
Writes features to CityGML 2.0 files.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CityGmlWriter Parameters",
  "description": "Configuration for writing features to CityGML 2.0 files.",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "output": {
      "title": "Output File",
      "description": "Output path or expression for the CityGML file to create.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "prettyPrint": {
      "title": "Pretty Print",
      "description": "Whether to indent the output for readability. Defaults to true.",
      "default": true,
      "type": [
        "boolean",
        "null"
      ]
    },
    "lodFilter": {
      "title": "LOD Filter",
      "description": "Levels of detail to include, such as [0, 1, 2]. If empty, all levels are included.",
      "default": null,
      "type": [
        "array",
        "null"
      ],
      "items": {
        "type": "integer",
        "format": "uint8",
        "minimum": 0.0
      }
    },
    "epsgCode": {
      "title": "EPSG Code",
      "description": "EPSG code of the coordinate reference system to declare in the output.",
      "default": null,
      "type": [
        "integer",
        "null"
      ],
      "format": "uint32",
      "minimum": 0.0
    }
  }
}
```
### Input Ports
* features
### Output Ports
### Category
* Output

## Clipper
### Type
* processor
### Description
Clips candidate features to the boundary geometry, separating the results into inside and outside portions.
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

## Closed Curve Filter
### Type
* processor
### Description
Filter LineString Features by Closed/Open Status
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* closed
* open
* rejected
### Category
* Geometry

## Convex Hull Accumulator
### Type
* processor
### Description
Generate Convex Hull Polygons from Grouped Features
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Convex Hull Accumulator Parameters",
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
* features
### Output Ports
* features
* rejected
### Category
* Geometry

## Coordinate Extractor
### Type
* processor
### Description
Extracts coordinates from geometry vertices into feature attributes
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Coordinate Extractor Parameters",
  "type": "object",
  "required": [
    "mode"
  ],
  "properties": {
    "mode": {
      "title": "Extraction Mode",
      "description": "How to extract coordinates from geometry vertices.",
      "allOf": [
        {
          "$ref": "#/definitions/CoordinateExtractionMode"
        }
      ]
    },
    "defaultZValue": {
      "title": "Default Z Value",
      "description": "Z value to use for 2D geometries that have no Z coordinate.",
      "type": [
        "number",
        "null"
      ],
      "format": "double"
    }
  },
  "definitions": {
    "CoordinateExtractionMode": {
      "description": "Extraction mode: determines how coordinates are output.",
      "oneOf": [
        {
          "description": "Extract all vertices into a list attribute.",
          "type": "object",
          "required": [
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "allCoordinates"
              ]
            },
            "coordinatesListName": {
              "title": "Coordinates List Name",
              "description": "Name of the list attribute that will store coordinate objects (default: \"_indices\")",
              "default": "_indices",
              "allOf": [
                {
                  "$ref": "#/definitions/Attribute"
                }
              ]
            }
          }
        },
        {
          "description": "Extract a single vertex by index.",
          "type": "object",
          "required": [
            "coordinateIndex",
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "specifyCoordinate"
              ]
            },
            "coordinateIndex": {
              "title": "Coordinate Index",
              "description": "Index of the coordinate to extract. 0 = first vertex, negative values count from end (-1 = last).",
              "type": "integer",
              "format": "int64"
            },
            "xAttribute": {
              "title": "X Attribute Name",
              "description": "Name of the X coordinate attribute (default: \"_x\")",
              "default": "_x",
              "allOf": [
                {
                  "$ref": "#/definitions/Attribute"
                }
              ]
            },
            "yAttribute": {
              "title": "Y Attribute Name",
              "description": "Name of the Y coordinate attribute (default: \"_y\")",
              "default": "_y",
              "allOf": [
                {
                  "$ref": "#/definitions/Attribute"
                }
              ]
            },
            "zAttribute": {
              "title": "Z Attribute Name",
              "description": "Name of the Z coordinate attribute (default: \"_z\")",
              "default": "_z",
              "allOf": [
                {
                  "$ref": "#/definitions/Attribute"
                }
              ]
            }
          }
        }
      ]
    },
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* rejected
### Category
* Geometry

## Coordinate Frame Reprojector
### Type
* processor
### Description
Reprojects geometry between coordinate reference systems and converts between a CRS and a Euclidean frame.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Coordinate Frame Reprojector Parameters",
  "description": "Reproject geometry across coordinate reference systems and convert between a CRS and a Euclidean frame. Converting across the Euclidean/CRS boundary reinterprets coordinates as-is: values and ring winding are left unchanged, so orientation follows the destination frame's axis order.\n\nReprojecting across coordinate reference systems takes 2D geometry lying at a single elevation into 3D.",
  "type": "object",
  "required": [
    "destinationFrame"
  ],
  "properties": {
    "destinationFrame": {
      "title": "Destination Frame",
      "description": "Coordinate frame to convert geometry into. Choosing a CRS also requires its EPSG code.",
      "allOf": [
        {
          "$ref": "#/definitions/DestinationFrame"
        }
      ]
    },
    "basePointSource": {
      "title": "Base Point",
      "description": "How coordinates bridge the Euclidean/CRS boundary.",
      "default": {
        "type": "asIs"
      },
      "allOf": [
        {
          "$ref": "#/definitions/BasePoint"
        }
      ]
    }
  },
  "definitions": {
    "DestinationFrame": {
      "description": "The destination coordinate frame, carrying the input each kind needs.",
      "oneOf": [
        {
          "title": "CRS",
          "description": "Reproject to a coordinate reference system identified by an EPSG code.",
          "type": "object",
          "required": [
            "crs"
          ],
          "properties": {
            "crs": {
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            }
          },
          "additionalProperties": false
        },
        {
          "title": "Euclidean",
          "description": "Convert to a non-georeferenced Euclidean frame. This is the frame the planar geometry operations work in, so it is the on-ramp for actions that require flat 2D input.",
          "type": "string",
          "enum": [
            "euclidean"
          ]
        }
      ]
    },
    "BasePoint": {
      "description": "How coordinates bridge the Euclidean/CRS boundary, carrying the input each mode needs. Ignored for a pure CRS-to-CRS reprojection.",
      "oneOf": [
        {
          "title": "As Is",
          "description": "Reinterpret coordinate values unchanged across the boundary.",
          "type": "object",
          "required": [
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "asIs"
              ]
            }
          }
        },
        {
          "title": "Value",
          "description": "Offset by a base point given as an expression evaluating to `[x, y, z]`.",
          "type": "object",
          "required": [
            "basePoint",
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "value"
              ]
            },
            "basePoint": {
              "title": "Base Point",
              "description": "Expression evaluating to an `[x, y, z]` origin in CRS space, in the CRS's declared axis order.",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            }
          }
        },
        {
          "title": "From Port",
          "description": "Offset by a base point taken from the base-point input port, matched to each feature by a key.",
          "type": "object",
          "required": [
            "matchKey",
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "fromPort"
              ]
            },
            "matchKey": {
              "title": "Match Key",
              "description": "Expression identifying which base-point feature applies to a given feature. Evaluated against both streams.",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            }
          }
        }
      ]
    }
  }
}
```
### Input Ports
* features
* base-point
### Output Ports
* features
* rejected
### Category
* Geometry

## Date Time Converter
### Type
* processor
### Description
Reads a datetime from an attribute and rewrites it in another format, either as a typed datetime value or as a formatted string or Unix timestamp.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Date Time Converter Parameters",
  "description": "Selects the attribute to convert, how to interpret its current value, and the format to write back.",
  "type": "object",
  "required": [
    "attribute"
  ],
  "properties": {
    "attribute": {
      "title": "Source Attribute",
      "description": "Attribute holding the datetime to convert. Features that do not carry it pass through unchanged.",
      "type": "string"
    },
    "inputFormat": {
      "title": "Input Format",
      "description": "How to interpret the existing value. Leave unset to detect it from the value itself.",
      "default": "auto",
      "allOf": [
        {
          "$ref": "#/definitions/DateTimeInputFormat"
        }
      ]
    },
    "outputFormat": {
      "title": "Output Format",
      "description": "Format to write back. Leave unset to store a typed datetime value; choose any other format to write a string or a number instead.",
      "default": "auto",
      "allOf": [
        {
          "$ref": "#/definitions/DateTimeOutputFormat"
        }
      ]
    },
    "outputAttribute": {
      "title": "Output Attribute",
      "description": "Attribute to write the result to, leaving the source untouched. Defaults to overwriting the source attribute.",
      "default": null,
      "type": [
        "string",
        "null"
      ]
    }
  },
  "definitions": {
    "DateTimeInputFormat": {
      "description": "Input format options for Date Time Converter",
      "oneOf": [
        {
          "title": "Detect Automatically",
          "description": "Recognises the value from its own shape. A bare number is always read as Unix seconds — choose Unix Milliseconds explicitly for millisecond timestamps.",
          "type": "string",
          "enum": [
            "auto"
          ]
        },
        {
          "title": "RFC 3339",
          "description": "Internet date and time format, the profile of ISO 8601 used by most APIs.",
          "type": "string",
          "enum": [
            "rfc3339"
          ]
        },
        {
          "title": "Unix Seconds",
          "description": "Seconds elapsed since 1970-01-01 UTC.",
          "type": "string",
          "enum": [
            "unixS"
          ]
        },
        {
          "title": "Unix Milliseconds",
          "description": "Milliseconds elapsed since 1970-01-01 UTC.",
          "type": "string",
          "enum": [
            "unixMs"
          ]
        },
        {
          "title": "Date Only",
          "description": "Calendar date with no time part, as YYYY-MM-DD.",
          "type": "string",
          "enum": [
            "date"
          ]
        },
        {
          "title": "Custom Pattern",
          "description": "Field-by-field pattern, for values none of the fixed formats match.",
          "type": "object",
          "required": [
            "custom"
          ],
          "properties": {
            "custom": {
              "type": "string"
            }
          },
          "additionalProperties": false
        }
      ]
    },
    "DateTimeOutputFormat": {
      "description": "Output format options for Date Time Converter",
      "oneOf": [
        {
          "title": "Typed Datetime",
          "description": "Stores a datetime value rather than text, keeping it comparable and sortable downstream.",
          "type": "string",
          "enum": [
            "auto"
          ]
        },
        {
          "title": "RFC 3339",
          "description": "Writes a string in the internet date and time format, the profile of ISO 8601 used by most APIs.",
          "type": "string",
          "enum": [
            "rfc3339"
          ]
        },
        {
          "title": "Unix Seconds",
          "description": "Writes a number: seconds elapsed since 1970-01-01 UTC.",
          "type": "string",
          "enum": [
            "unixS"
          ]
        },
        {
          "title": "Unix Milliseconds",
          "description": "Writes a number: milliseconds elapsed since 1970-01-01 UTC.",
          "type": "string",
          "enum": [
            "unixMs"
          ]
        },
        {
          "title": "Date Only",
          "description": "Writes a string holding just the calendar date, as YYYY-MM-DD.",
          "type": "string",
          "enum": [
            "date"
          ]
        },
        {
          "title": "Custom Pattern",
          "description": "Writes a string built from a field-by-field pattern.",
          "type": "object",
          "required": [
            "custom"
          ],
          "properties": {
            "custom": {
              "type": "string"
            }
          },
          "additionalProperties": false
        }
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* failed
### Category
* Attribute

## Dimension Filter
### Type
* processor
### Description
Routes features to output ports based on the number of geometry dimensions.
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* 2d
* 3d
* rejected
### Category
* Filter

## Directory Decompressor
### Type
* processor
### Description
Decompresses zip and 7z archives referenced by feature attributes, replacing each attribute value with the path of the directory holding the extracted files.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Directory Decompressor Parameters",
  "description": "Configures which attributes hold archives and how deeply the extracted directory is unwrapped.",
  "type": "object",
  "required": [
    "archiveAttributes"
  ],
  "properties": {
    "archiveAttributes": {
      "title": "Archive Attributes",
      "description": "Attributes holding the path of a `.zip`, `.7z`, or `.7zip` archive. Each is replaced with the extracted directory path; attributes holding any other value are left unchanged.",
      "type": "array",
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "findDeepestSingleFolder": {
      "title": "Find Deepest Single Folder",
      "description": "Keeps unwrapping while the extracted directory contains nothing but a single subdirectory, stopping at the first directory with multiple entries or a file. When disabled, only one level of single-folder nesting is unwrapped.",
      "type": [
        "boolean",
        "null"
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
* features
### Output Ports
* features
### Category
* File

## Dissolver
### Type
* processor
### Description
Merges area features that share the same grouping attribute values into one feature per group, combining their geometries and dropping the boundaries between them. Inputs must be flat 2D areas sharing one coordinate frame; place a Two Dimension Forcer or a Coordinate Frame Reprojector upstream to flatten or unify them.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Dissolver Parameters",
  "description": "Sets which features are dissolved together, how closely their vertices must line up, and which of their attributes the merged feature keeps.",
  "type": "object",
  "properties": {
    "groupBy": {
      "title": "Group By Attributes",
      "description": "Attributes whose values decide which features dissolve together — features matching on all of them are merged into one. When omitted, every feature forms a single group.",
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
      "description": "Distance below which two vertices are treated as the same point when merging geometries, in the unit of the input's coordinate frame. Defaults to zero, which merges only exactly coincident vertices and can leave slivers between edges that nearly meet.",
      "type": [
        "number",
        "null"
      ],
      "format": "double"
    },
    "attributeAccumulation": {
      "title": "Attribute Accumulation",
      "description": "Which attributes the merged feature keeps.",
      "default": "useOneFeature",
      "allOf": [
        {
          "$ref": "#/definitions/AttributeAccumulationStrategy"
        }
      ]
    }
  },
  "definitions": {
    "Attribute": {
      "type": "string"
    },
    "AttributeAccumulationStrategy": {
      "title": "Attribute Accumulation Strategy",
      "description": "Which attributes a merged feature keeps.",
      "oneOf": [
        {
          "title": "Drop Incoming Attributes",
          "description": "Keeps only the grouping attributes, discarding everything else.",
          "type": "string",
          "enum": [
            "dropAttributes"
          ]
        },
        {
          "title": "Merge Incoming Attributes",
          "description": "Keeps every attribute from every feature in the group. Where features disagree on a value, all the differing values are collected into an array.",
          "type": "string",
          "enum": [
            "mergeAttributes"
          ]
        },
        {
          "title": "Use Attributes From One Feature",
          "description": "Keeps the attributes of a single feature from the group and discards the rest.",
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
* features
### Output Ports
* features
* rejected
### Category
* Geometry

## Echo Processor
### Type
* processor
### Description
Echoes features to logs and passes them through unchanged.
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* features
### Category
* Debug

## Echo Sink
### Type
* sink
### Description
Echoes features to logs and discards them.
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
### Category
* Debug

## Elevation Extractor
### Type
* processor
### Description
Extracts the elevation of a geometry's first vertex into an attribute. A geometry carrying no elevation passes through without it.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Elevation Extractor Parameters",
  "description": "Sets where the extracted elevation is stored.",
  "type": "object",
  "required": [
    "outputAttribute"
  ],
  "properties": {
    "outputAttribute": {
      "title": "Output Attribute",
      "description": "Attribute the elevation is written to. It is left unwritten when the geometry carries no elevation.",
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
* features
### Output Ports
* features
### Category
* Geometry

## Excel Writer
### Type
* sink
### Description
Writes each feature as a row of an .xlsx worksheet, one column per attribute. An attribute named after another with a `.formula` or `.hyperlink` suffix fills that column's cell with a formula or a link instead of a value.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Excel Writer Parameters",
  "description": "Sets where the workbook is written and what its worksheet is called.",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "output": {
      "title": "Output Path",
      "description": "Path the .xlsx file is written to. An expression is evaluated per feature, so features that resolve to different paths are written to separate workbooks.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "sheetName": {
      "title": "Sheet Name",
      "description": "Name of the worksheet the rows are written to. Defaults to `Sheet1`.",
      "default": "Sheet1",
      "type": "string"
    }
  }
}
```
### Input Ports
* features
### Output Ports
### Category
* Output

## Extruder
### Type
* processor
### Description
Extrudes a polygon geometry vertically by a given distance to produce a solid geometry.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Extruder Parameters",
  "description": "Configure how far each polygon is extruded.",
  "type": "object",
  "required": [
    "distance"
  ],
  "properties": {
    "distance": {
      "title": "Distance",
      "description": "Height to extrude the polygon by, as a constant or an expression evaluated per feature.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* rejected
### Category
* Geometry

## Feature CityGML 2 Reader
### Type
* processor
### Description
Reads CityGML 2.0 files, resolving gml:id references and xlink:href links across files.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Feature CityGML 2 Reader Parameters",
  "type": "object",
  "required": [
    "dataset"
  ],
  "properties": {
    "dataset": {
      "title": "Dataset",
      "description": "Path expression resolving to the CityGML 2.0 file to read.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "extractTags": {
      "title": "Extract Tags",
      "description": "Feature type names to flatten as individual features. Accepts qualified (`bldg:Building`), local (`Building`), or Clark notation (`{http://…}Building`). Empty means emit all top-level city objects unchanged.",
      "default": [],
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    "keepAttributes": {
      "title": "Keep Attributes",
      "description": "When false, XML attributes (`@`-prefixed entries such as `@gml:id`, `@codeSpace`) are dropped from parsed features. Defaults to true.",
      "default": true,
      "type": "boolean"
    },
    "flattenSingleChildObjects": {
      "title": "Flatten Single-Child Object Nodes",
      "description": "When true, a wrapper element whose only content is a single child element is dropped: the child is hoisted up and keyed by its own tag name, always wrapped in an array. Defaults to false.",
      "default": false,
      "type": "boolean"
    },
    "flattenMeasureTypes": {
      "title": "Flatten Measure Types",
      "description": "When true, elements with a single `uom` attribute and numeric text content are converted to a number value, with the unit stored as a sibling `{name}_uom` key. Defaults to false.",
      "default": false,
      "type": "boolean"
    },
    "cityGmlAttributesKey": {
      "title": "City GML Attributes Key",
      "description": "When set, parsed CityGML attributes are nested under this key in the output feature. When null, attributes are emitted at the top level. Defaults to null.",
      "default": null,
      "type": [
        "string",
        "null"
      ]
    },
    "inheritInputAttributes": {
      "title": "Inherit Input Attributes",
      "description": "When true, the input feature's attributes are merged into every feature parsed from its file. Defaults to true.",
      "default": true,
      "type": "boolean"
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Input

## Feature CityGML 3 Reader
### Type
* processor
### Description
Reads the CityGML 3.0 file each incoming feature points at, resolving gml:id and xlink:href references across every file read. The attributes of the feature naming a file are carried onto the features parsed from it.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Feature CityGML 3 Reader Parameters",
  "description": "Which file to read, and how its elements become feature attributes.",
  "type": "object",
  "required": [
    "dataset"
  ],
  "properties": {
    "dataset": {
      "title": "Dataset",
      "description": "Path expression resolving to the CityGML 3.0 file to read.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "extractTags": {
      "title": "Extract Tags",
      "description": "Feature type names to flatten as individual features. Accepts qualified (`bldg:Building`), local (`Building`), or Clark notation (`{http://…}Building`). Empty means emit all top-level city objects unchanged.",
      "default": [],
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    "keepAttributes": {
      "title": "Keep Attributes",
      "description": "When false, XML attributes (`@`-prefixed entries such as `@gml:id`, `@codeSpace`) are dropped from parsed features. Defaults to true.",
      "default": true,
      "type": "boolean"
    },
    "flattenSingleChildObjects": {
      "title": "Flatten Single-Child Object Nodes",
      "description": "When true, a wrapper element whose only content is a single child element is dropped: the child is hoisted up and keyed by its own tag name, always wrapped in an array. Defaults to false.",
      "default": false,
      "type": "boolean"
    },
    "flattenLeafAttributes": {
      "title": "Flatten Leaf Attributes",
      "description": "Attribute names (e.g. `uom`) that mark a leaf for collapsing: an element with exactly one XML attribute in this list, no child elements, and numeric text content is converted to a number value, with the attribute's value stored as a sibling `{name}_{attribute}` key. Empty (the default) disables this.",
      "default": [],
      "type": "array",
      "items": {
        "type": "string"
      }
    },
    "cityGmlAttributesKey": {
      "title": "City GML Attributes Key",
      "description": "When set, parsed CityGML attributes are nested under this key in the output feature. When null, attributes are emitted at the top level. Defaults to null.",
      "default": null,
      "type": [
        "string",
        "null"
      ]
    },
    "inheritInputAttributes": {
      "title": "Inherit Input Attributes",
      "description": "When true, the input feature's attributes are merged into every feature parsed from its file. Defaults to true.",
      "default": true,
      "type": "boolean"
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Feature

## Feature CityGML Reader
### Type
* processor
### Description
Reads CityGML features from a file path referenced by the incoming feature, optionally extracting nested child city objects as separate features.
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
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "flatten": {
      "title": "Flatten",
      "description": "Whether to flatten the hierarchical structure of the CityGML data",
      "type": [
        "boolean",
        "null"
      ]
    },
    "codelistsPath": {
      "title": "Codelists Path",
      "description": "Optional path to the codelists directory for resolving codelist values",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Input

## Feature Counter
### Type
* processor
### Description
Assigns a sequential number to each feature, stored in an attribute and optionally grouped by attribute values.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Feature Counter Parameters",
  "description": "Configure how features are numbered and grouped, and where to store the number",
  "type": "object",
  "required": [
    "outputAttribute"
  ],
  "properties": {
    "outputAttribute": {
      "title": "Count Attribute",
      "description": "Name of the attribute where the count will be stored",
      "type": "string"
    },
    "countStart": {
      "title": "Start Count",
      "description": "Value assigned to the first feature",
      "default": 1,
      "type": "integer",
      "format": "uint64",
      "minimum": 0.0
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
* features
### Output Ports
* features
### Category
* Debug

## Feature Creator
### Type
* source
### Description
Creates features from a script expression that returns one or more attribute maps.
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
      "description": "Script expression that returns a map (single feature) or an array of maps (multiple features). Each map holds feature attributes as key-value pairs.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
### Output Ports
* features
### Category
* Input

## Feature Duplicate Filter
### Type
* processor
### Description
Forwards the first feature carrying each distinct value and separates out the ones that repeat it. Features are compared on their whole content unless the attributes to compare are named.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Feature Duplicate Filter Parameters",
  "description": "Which features count as repeats of one another.",
  "type": "object",
  "properties": {
    "filterBy": {
      "title": "Filter Attributes",
      "description": "Attributes whose combined values identify a repeat: the first feature carrying a given combination is forwarded and later ones are separated out. An attribute that is absent counts as part of the combination, so it is not the same as one holding an empty value. When omitted, features are compared on their whole content instead — every attribute and their geometry.",
      "default": null,
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
* features
### Output Ports
* features
* duplicate
### Category
* Feature

## Feature File Path Extractor
### Type
* processor
### Description
Expands a dataset path into one feature per file, listing directories recursively and optionally extracting zip and 7z archives. Each emitted feature carries the file's path, name, and extension attributes alongside the attributes of the incoming feature.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Feature File Path Extractor Parameters",
  "description": "Configures which dataset is expanded into file features and whether archives are extracted.",
  "type": "object",
  "required": [
    "sourceDataset"
  ],
  "properties": {
    "sourceDataset": {
      "title": "Source Dataset",
      "description": "Expression evaluating to the path or URL of the file, directory, or archive to expand. A directory is listed recursively; any other path yields a single feature.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "extractArchive": {
      "title": "Extract Archive",
      "description": "Extracts the source dataset when it is a `.zip`, `.7z`, or `.7zip` archive and emits one feature per extracted file. When disabled, the archive itself is emitted as a single path.",
      "default": false,
      "type": "boolean"
    },
    "destPrefix": {
      "title": "Destination Prefix",
      "description": "Subdirectory created under the temporary extraction directory to hold the extracted files. Applies only when an archive is extracted.",
      "type": [
        "string",
        "null"
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Feature

## Feature Filter
### Type
* processor
### Description
Routes features to named output ports based on user-defined filter conditions.
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
          "title": "Condition Expression",
          "description": "Boolean expression evaluated against each feature; features for which it returns true are routed to this condition's output port.",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "outputPort": {
          "title": "Output Port",
          "description": "Name of the output port that receives features matching this condition.",
          "allOf": [
            {
              "$ref": "#/definitions/Port"
            }
          ]
        }
      }
    },
    "Port": {
      "type": "string"
    }
  }
}
```
### Input Ports
* features
### Output Ports
* unfiltered
### Category
* Filter

## Feature GeoJSON Writer
### Type
* processor
### Description
Writes the features it receives to a GeoJSON file per resolved output path, then emits one feature per file written, carrying that file's path.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Feature GeoJSON Writer Parameters",
  "description": "Where the GeoJSON is written, and what is declared about its coordinates.",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "output": {
      "title": "Output",
      "description": "Path (or expression evaluated per feature) of the GeoJSON file to write. Features sharing a resolved path are written to the same file, so the expression can split features across files by attribute value.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "writeCrs": {
      "title": "Write CRS",
      "description": "Whether to declare the coordinate reference system of the written coordinates in a legacy GeoJSON 2008 `crs` member. Defaults to false; enable it when the coordinates are not WGS84 longitude / latitude and the consumer reads that member.",
      "default": false,
      "type": "boolean"
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Feature

## Feature Joiner
### Type
* processor
### Description
Joins requestor and supplier features based on matching attribute values, with configurable join types.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Feature Joiner Parameters",
  "description": "Configuration for joining requestor and supplier features based on matching attributes or expressions.",
  "type": "object",
  "required": [
    "joinType"
  ],
  "properties": {
    "joinType": {
      "title": "Join Type",
      "description": "How unmatched features are handled: inner, left, or full.",
      "allOf": [
        {
          "$ref": "#/definitions/JoinType"
        }
      ]
    },
    "requestorAttribute": {
      "title": "Requestor Attributes",
      "description": "Attributes from requestor features to use for matching (alternative to requestorAttributeValue).",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "supplierAttribute": {
      "title": "Supplier Attributes",
      "description": "Attributes from supplier features to use for matching (alternative to supplierAttributeValue).",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "requestorAttributeValue": {
      "title": "Requestor Attribute Value",
      "description": "Expression to evaluate for requestor feature matching values (alternative to requestorAttribute).",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "supplierAttributeValue": {
      "title": "Supplier Attribute Value",
      "description": "Expression to evaluate for supplier feature matching values (alternative to supplierAttribute).",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "conflictResolution": {
      "title": "Conflict Resolution",
      "description": "How to resolve conflicts when requestor and supplier features share the same attribute.",
      "anyOf": [
        {
          "$ref": "#/definitions/ConflictResolution"
        },
        {
          "type": "null"
        }
      ]
    }
  },
  "definitions": {
    "JoinType": {
      "oneOf": [
        {
          "title": "Inner",
          "description": "Only emits features where a match exists.",
          "type": "string",
          "enum": [
            "inner"
          ]
        },
        {
          "title": "Left",
          "description": "Emits all requestor features, matched or not.",
          "type": "string",
          "enum": [
            "left"
          ]
        },
        {
          "title": "Full",
          "description": "Emits all features from both sides.",
          "type": "string",
          "enum": [
            "full"
          ]
        }
      ]
    },
    "Attribute": {
      "type": "string"
    },
    "ConflictResolution": {
      "oneOf": [
        {
          "title": "Requestor Wins",
          "description": "Requestor attributes win on conflict.",
          "type": "string",
          "enum": [
            "requestorWins"
          ]
        },
        {
          "title": "Supplier Wins",
          "description": "Supplier attributes win on conflict (default).",
          "type": "string",
          "enum": [
            "supplierWins"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* requestor
* supplier
### Output Ports
* joined
* unjoined-requestor
* unjoined-supplier
### Category
* Merge

## Feature LOD Filter
### Type
* processor
### Description
Filters features by Level of Detail (LOD), emitting each to the matching LOD output port.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Feature LOD Filter Parameters",
  "description": "Configuration for filtering features based on Level of Detail (LOD).",
  "type": "object",
  "required": [
    "filterKey"
  ],
  "properties": {
    "filterKey": {
      "title": "Filter Key",
      "description": "Attribute whose value groups features; the maximum available LOD is determined within each group.",
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
* features
### Output Ports
* max-lod0
* max-lod1
* max-lod2
* max-lod3
* max-lod4
* unfiltered
### Category
* Filter

## Feature Merger
### Type
* processor
### Description
Merges requestor and supplier features based on matching attribute values.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Feature Merger Parameters",
  "description": "Configuration for merging requestor and supplier features based on matching attributes or expressions.",
  "type": "object",
  "properties": {
    "requestorAttribute": {
      "title": "Requestor Attributes",
      "description": "Attributes from requestor features to use for matching (alternative to requestorAttributeValue).",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "supplierAttribute": {
      "title": "Supplier Attributes",
      "description": "Attributes from supplier features to use for matching (alternative to supplierAttributeValue).",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "requestorAttributeValue": {
      "title": "Requestor Attribute Value",
      "description": "Expression to evaluate for requestor feature matching values (alternative to requestorAttribute).",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "supplierAttributeValue": {
      "title": "Supplier Attribute Value",
      "description": "Expression to evaluate for supplier feature matching values (alternative to supplierAttribute).",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "completeGrouped": {
      "title": "Complete Grouped",
      "description": "Whether to complete grouped features before processing the next group.",
      "type": [
        "boolean",
        "null"
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
* requestor
* supplier
### Output Ports
* merged
* unmerged
### Category
* Merge

## Feature Reader
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
        "format": {
          "type": "string",
          "enum": [
            "csv"
          ]
        },
        "encoding": {
          "title": "Character Encoding",
          "description": "Character encoding for the CSV file. If not specified, defaults to UTF-8. Supported: UTF-8, Shift-JIS, EUC-JP, Windows-1252, ISO-8859-1, etc.",
          "type": [
            "string",
            "null"
          ]
        },
        "dataset": {
          "title": "Dataset",
          "description": "Path or expression to the dataset file to be read",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr",
                "string"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "offset": {
          "description": "The offset of the first row to read",
          "type": [
            "integer",
            "null"
          ],
          "format": "uint",
          "minimum": 0.0
        },
        "headerRows": {
          "title": "Header Row Count",
          "description": "Number of consecutive rows that make up the header (default: 1). When 0, no header rows are read and column names are auto-generated as \"column1\", \"column2\", etc. When greater than 1, column names are formed by joining non-empty values from each header row with \"_\".",
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
        "format": {
          "type": "string",
          "enum": [
            "tsv"
          ]
        },
        "encoding": {
          "title": "Character Encoding",
          "description": "Character encoding for the TSV file. If not specified, defaults to UTF-8. Supported: UTF-8, Shift-JIS, EUC-JP, Windows-1252, ISO-8859-1, etc.",
          "type": [
            "string",
            "null"
          ]
        },
        "dataset": {
          "title": "Dataset",
          "description": "Path or expression to the dataset file to be read",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr",
                "string"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "offset": {
          "description": "The offset of the first row to read",
          "type": [
            "integer",
            "null"
          ],
          "format": "uint",
          "minimum": 0.0
        },
        "headerRows": {
          "title": "Header Row Count",
          "description": "Number of consecutive rows that make up the header (default: 1). When 0, no header rows are read and column names are auto-generated as \"column1\", \"column2\", etc. When greater than 1, column names are formed by joining non-empty values from each header row with \"_\".",
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
        "format": {
          "type": "string",
          "enum": [
            "json"
          ]
        },
        "dataset": {
          "title": "Dataset",
          "description": "Path or expression to the dataset file to be read",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr",
                "string"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        }
      }
    }
  ]
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Feature

## Feature Sorter
### Type
* processor
### Description
Sorts features based on specified attributes in ascending or descending order.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Feature Sorter Parameters",
  "description": "Configuration for sorting features based on attribute values.",
  "type": "object",
  "required": [
    "attributes",
    "order"
  ],
  "properties": {
    "attributes": {
      "title": "Sort Attributes",
      "description": "Attributes to sort by; earlier attributes take precedence.",
      "type": "array",
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "order": {
      "title": "Sort Order",
      "description": "Whether features are sorted in ascending or descending order.",
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
* features
### Output Ports
* features
### Category
* Merge

## Feature Transformer
### Type
* processor
### Description
Replaces each feature's attributes with the map returned by one or more expressions, applied in order. Geometry is passed through unchanged.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Feature Transformer Parameters",
  "description": "Configures the expressions that build each feature's new attributes.",
  "type": "object",
  "required": [
    "transformers"
  ],
  "properties": {
    "transformers": {
      "title": "Transformations",
      "description": "Expressions applied in order, each one reading the attributes produced by the previous.",
      "type": "array",
      "items": {
        "$ref": "#/definitions/Transform"
      }
    }
  },
  "definitions": {
    "Transform": {
      "type": "object",
      "required": [
        "expr"
      ],
      "properties": {
        "expr": {
          "title": "Expression",
          "description": "Expression over `attributes` and `variables` returning a map that becomes the feature's complete attribute set. A result that is not a map, or an expression that fails to evaluate, leaves the attributes unchanged.",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Transform

## Feature Type Filter
### Type
* processor
### Description
Filters CityGML features by their feature type.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Feature Type Filter Parameters",
  "description": "Configuration for filtering features based on their feature type.",
  "type": "object",
  "required": [
    "targetTypes"
  ],
  "properties": {
    "targetTypes": {
      "title": "Target Feature Types",
      "description": "List of CityGML feature type names to match, such as \"bldg:Building\" or \"tran:TrafficArea\".",
      "type": "array",
      "items": {
        "type": "string"
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* unfiltered
### Category
* Filter

## Feature Writer
### Type
* processor
### Description
Writes features from various formats
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Feature Writer Parameters",
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
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr",
                "string"
              ]
            },
            "value": {
              "type": "string"
            }
          }
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
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr",
                "string"
              ]
            },
            "value": {
              "type": "string"
            }
          }
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
        "format": {
          "type": "string",
          "enum": [
            "json"
          ]
        },
        "output": {
          "title": "Output path",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr",
                "string"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "converter": {
          "type": [
            "object",
            "null"
          ],
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        }
      }
    },
    {
      "title": "CityGmlWriter Parameters",
      "description": "Configuration for writing features in CityGML 2.0 format.",
      "type": "object",
      "required": [
        "format",
        "output"
      ],
      "properties": {
        "format": {
          "type": "string",
          "enum": [
            "citygml"
          ]
        },
        "output": {
          "title": "Output path",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr",
                "string"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "lodFilter": {
          "description": "LOD levels to include (e.g., [0, 1, 2]). If empty, includes all LODs.",
          "default": null,
          "type": [
            "array",
            "null"
          ],
          "items": {
            "type": "integer",
            "format": "uint8",
            "minimum": 0.0
          }
        },
        "epsgCode": {
          "description": "EPSG code for coordinate reference system",
          "default": null,
          "type": [
            "integer",
            "null"
          ],
          "format": "uint32",
          "minimum": 0.0
        },
        "prettyPrint": {
          "description": "Whether to format output with indentation (default: true)",
          "default": true,
          "type": [
            "boolean",
            "null"
          ]
        }
      }
    }
  ]
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Feature

## File Path Extractor
### Type
* source
### Description
Extracts file paths from directories or archives, creating features for each discovered file.
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
    "sourceDataset": {
      "title": "Source Dataset",
      "description": "Path or expression pointing to the source directory or archive file",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "extractArchive": {
      "title": "Extract Archive",
      "description": "When enabled, archive files (.zip, .7z) are extracted and a feature is emitted for each contained file; when disabled, the archive is emitted as a single file path without extraction.",
      "type": "boolean"
    }
  }
}
```
### Input Ports
### Output Ports
* features
### Category
* Input

## File Property Extractor
### Type
* processor
### Description
Inspects the file or directory at a path held in a feature attribute and adds its type, size, and access, modification, and creation timestamps as attributes. Directory size is the total size of the files it contains, counted recursively.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "File Property Extractor Parameters",
  "description": "Configures which attribute holds the path to inspect.",
  "type": "object",
  "required": [
    "filePathAttribute"
  ],
  "properties": {
    "filePathAttribute": {
      "title": "File Path Attribute",
      "description": "Name of the attribute holding the path of the file or directory to inspect. Paths that do not exist, and symbolic links, are not inspected.",
      "type": "string"
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* rejected
### Category
* File

## Footprint Replacer
### Type
* processor
### Description
Replaces a feature's geometry with its footprint: the dissolved projection of its faces onto the horizontal plane or a custom plane.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Footprint Replacer Parameters",
  "description": "Configure the plane the geometry is projected onto. The geometry must be in a Euclidean frame or a coordinate reference system in linear units; faces smaller than 1e-6 square units after projection are dropped.",
  "type": "object",
  "properties": {
    "projectionPlane": {
      "title": "Projection Plane",
      "description": "Plane the geometry is projected onto. Defaults to the horizontal plane.",
      "default": {
        "type": "horizontal"
      },
      "allOf": [
        {
          "$ref": "#/definitions/ProjectionPlane"
        }
      ]
    }
  },
  "definitions": {
    "ProjectionPlane": {
      "description": "The plane the footprint is projected onto.",
      "oneOf": [
        {
          "title": "Horizontal",
          "description": "Projects along the vertical axis onto the horizontal plane, dropping height. The footprint stays in the geometry's horizontal coordinate system.",
          "type": "object",
          "required": [
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "horizontal"
              ]
            }
          }
        },
        {
          "title": "Custom Plane",
          "description": "Projects along a plane's normal onto that plane. The footprint is expressed in in-plane coordinates anchored at the plane origin.",
          "type": "object",
          "required": [
            "normal",
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "custom"
              ]
            },
            "normal": {
              "title": "Normal",
              "description": "Expression evaluating to the plane normal `[x, y, z]` in the geometry's coordinate frame; any non-zero length.",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            },
            "origin": {
              "title": "Origin",
              "description": "Expression evaluating to the plane origin `[x, y, z]` in the geometry's coordinate frame. Defaults to `[0, 0, 0]`.",
              "default": null,
              "type": [
                "object",
                "null"
              ],
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            },
            "xAxis": {
              "title": "X Axis",
              "description": "Expression evaluating to a direction `[x, y, z]` whose in-plane component becomes the footprint's x axis. When omitted, the footprint's y axis is the in-plane direction closest to vertical.",
              "default": null,
              "type": [
                "object",
                "null"
              ],
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            }
          }
        }
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* footprint
* rejected
### Category
* Geometry

## GeoJSON Reader
### Type
* source
### Description
Reads geographic features from a GeoJSON FeatureCollection file.
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
      "description": "Expression that returns the path to the input file, either a literal path or a variable reference.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
### Output Ports
* features
### Category
* Input

## GeoJSON Writer
### Type
* sink
### Description
Writes features to GeoJSON files, optionally grouping them into separate files.
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
    "output": {
      "title": "Output File",
      "description": "Output path or expression for the GeoJSON file to create.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "groupBy": {
      "title": "Group By",
      "description": "Attributes to group features by, writing a separate file for each distinct group.",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "writeCrs": {
      "title": "Write CRS",
      "description": "Whether to declare the coordinate reference system of the written coordinates in a legacy GeoJSON 2008 `crs` member. Defaults to false; enable it when the coordinates are not WGS84 longitude / latitude and the consumer reads that member.",
      "default": false,
      "type": "boolean"
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
* features
### Output Ports
### Category
* Output

## GeoPackage Reader
### Type
* source
### Description
Reads vector features from GeoPackage (.gpkg) files, or the file's spatial reference and extension metadata.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GeoPackage Reader Parameters",
  "description": "Configures which content to read from the GeoPackage file and how geometries are produced.",
  "type": "object",
  "properties": {
    "readMode": {
      "title": "Read Mode",
      "description": "Which content to read from the GeoPackage file. Defaults to reading vector features.",
      "default": "features",
      "allOf": [
        {
          "$ref": "#/definitions/GeoPackageReadMode"
        }
      ]
    },
    "layerName": {
      "title": "Layer Name",
      "description": "Name of the layer to read. When omitted, every feature layer in the file is read.",
      "type": [
        "string",
        "null"
      ]
    },
    "force2D": {
      "title": "Force 2D",
      "description": "Drops the Z value from every geometry, producing 2D output.",
      "default": false,
      "type": "boolean"
    },
    "dataset": {
      "title": "File Path",
      "description": "Expression that returns the path to the input file, either a literal path or a variable reference.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  },
  "definitions": {
    "GeoPackageReadMode": {
      "oneOf": [
        {
          "title": "Features",
          "description": "Reads vector features, each carrying its geometry and attributes.",
          "type": "string",
          "enum": [
            "features"
          ]
        },
        {
          "title": "Metadata Only",
          "description": "Reads the file's spatial reference systems and registered extensions instead of its features. Emits one feature per metadata record.",
          "type": "string",
          "enum": [
            "metadataOnly"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
### Output Ports
* features
### Category
* Input

## GeoPackage Writer
### Type
* sink
### Description
Writes features to a GeoPackage (.gpkg) file.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GeoPackageWriter Parameters",
  "description": "Configuration for writing features to GeoPackage files.",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "output": {
      "title": "Output File",
      "description": "Output path or expression for the GeoPackage file to create.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "tableName": {
      "title": "Table Name",
      "description": "Name of the feature table to create, shown as the layer name in GIS clients. Defaults to \"features\".",
      "default": "features",
      "type": "string"
    },
    "srsId": {
      "title": "SRS ID",
      "description": "Spatial reference system identifier (EPSG code) recorded for the geometry. The writer tags this code without reprojecting, so it must match the coordinate system of the input data. Defaults to 4326 (WGS 84).",
      "default": 4326,
      "type": "integer",
      "format": "int32"
    },
    "geometryType": {
      "title": "Geometry Type",
      "description": "Geometry type declared for the table. Use Geometry for tables that hold mixed or unknown types. Defaults to Geometry.",
      "default": "geometry",
      "allOf": [
        {
          "$ref": "#/definitions/GeometryType"
        }
      ]
    },
    "overwrite": {
      "title": "Overwrite Existing File",
      "description": "Whether to overwrite the output file if it already exists. Defaults to false.",
      "default": false,
      "type": "boolean"
    },
    "geometryColumn": {
      "title": "Geometry Column",
      "description": "Name of the column that stores geometry. Defaults to \"geom\".",
      "default": "geom",
      "type": "string"
    },
    "createSpatialIndex": {
      "title": "Create Spatial Index",
      "description": "Whether to build an R-tree spatial index on the geometry column for faster queries. Defaults to true.",
      "default": true,
      "type": "boolean"
    }
  },
  "definitions": {
    "GeometryType": {
      "title": "Geometry Type",
      "description": "Geometry type declared for a GeoPackage feature table.",
      "oneOf": [
        {
          "title": "Point",
          "description": "A single point per feature.",
          "type": "string",
          "enum": [
            "point"
          ]
        },
        {
          "title": "Line String",
          "description": "A single connected sequence of points per feature.",
          "type": "string",
          "enum": [
            "lineString"
          ]
        },
        {
          "title": "Polygon",
          "description": "A single polygon, with optional holes, per feature.",
          "type": "string",
          "enum": [
            "polygon"
          ]
        },
        {
          "title": "Multi Point",
          "description": "A collection of points per feature.",
          "type": "string",
          "enum": [
            "multiPoint"
          ]
        },
        {
          "title": "Multi Line String",
          "description": "A collection of line strings per feature.",
          "type": "string",
          "enum": [
            "multiLineString"
          ]
        },
        {
          "title": "Multi Polygon",
          "description": "A collection of polygons per feature.",
          "type": "string",
          "enum": [
            "multiPolygon"
          ]
        },
        {
          "title": "Geometry (Mixed)",
          "description": "Any geometry type; use when the table holds mixed or unknown geometry types.",
          "type": "string",
          "enum": [
            "geometry"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
### Category
* Output

## Geometry Coercer
### Type
* processor
### Description
Coerces a feature's geometry into a different geometry type, rebuilding it as polylines, faces, or triangles.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Geometry Coercer Parameters",
  "description": "Which geometry type each feature is coerced into.",
  "type": "object",
  "required": [
    "targetType"
  ],
  "properties": {
    "targetType": {
      "title": "Target Type",
      "description": "Geometry type to re-represent each feature as. A feature the target does not apply to passes through unchanged.",
      "allOf": [
        {
          "$ref": "#/definitions/CoercionTarget"
        }
      ]
    }
  },
  "definitions": {
    "CoercionTarget": {
      "oneOf": [
        {
          "title": "Line String",
          "description": "Replaces every face with the polylines of its boundary rings, holes included.",
          "type": "string",
          "enum": [
            "lineString"
          ]
        },
        {
          "title": "Polygon",
          "description": "Rebuilds faces: a closed line string becomes the face it bounds, and a surface or a solid becomes the individual faces it is built from.",
          "type": "string",
          "enum": [
            "polygon"
          ]
        },
        {
          "title": "Triangular Mesh",
          "description": "Tessellates a face or a surface into triangles. A solid stays a solid, with its boundary triangulated.",
          "type": "string",
          "enum": [
            "triangularMesh"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Geometry

## Geometry Extractor
### Type
* processor
### Description
Serializes a feature's geometry to a compressed representation and stores it in an attribute.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Geometry Extractor Parameters",
  "description": "Configure where the serialized geometry is stored.",
  "type": "object",
  "required": [
    "outputAttribute"
  ],
  "properties": {
    "outputAttribute": {
      "title": "Output Attribute",
      "description": "Attribute to store the compressed geometry in. Geometry Replacer reads the same representation back onto a feature.",
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
* features
### Output Ports
* features
### Category
* Geometry

## Geometry Filter
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
      "title": "No Geometry",
      "description": "Separates the features that carry no geometry at all from the ones that do.",
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
      "title": "Geometry Type",
      "description": "Routes by the geometry family a feature belongs to: point, curve, surface, triangle or solid.",
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
    },
    {
      "title": "Detailed Geometry Type",
      "description": "Routes by the exact geometry type rather than the family, so a face, a surface mesh and a multi-surface each leave by their own port.",
      "type": "object",
      "required": [
        "filterType"
      ],
      "properties": {
        "filterType": {
          "type": "string",
          "enum": [
            "detailedGeometryType"
          ]
        }
      }
    }
  ]
}
```
### Input Ports
* features
### Output Ports
* unfiltered
* none
* point
* curve
* surface
* triangle
* solid
* multi-point
* point-cloud
* line-string
* multi-curve
* polygon
* multi-area
* face
* polygon-mesh
* triangular-mesh
* multi-surface
* csg
* multi-solid
* aggregate
### Category
* Geometry

## Geometry Part Extractor
### Type
* processor
### Description
Extracts the individual surfaces of a geometry, emitting each as a separate feature.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Geometry Part Extractor Parameters",
  "description": "Configure which kind of part is pulled out of each geometry.",
  "type": "object",
  "properties": {
    "geometryPartType": {
      "title": "Part Type",
      "description": "Kind of part to extract from the geometry.",
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
          "title": "Surface",
          "description": "Emits each surface of the geometry as a separate feature.",
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
* features
### Output Ports
* extracted
* remaining
* untouched
### Category
* Geometry

## Geometry Remover
### Type
* processor
### Description
Discards a feature's geometry, keeping its attributes.
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* features
### Category
* Geometry

## Geometry Replacer
### Type
* processor
### Description
Replaces a feature's geometry with the compressed geometry data stored in a named attribute.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Geometry Replacer Parameters",
  "description": "Configure which attribute holds the geometry that replaces the feature's current geometry.",
  "type": "object",
  "required": [
    "sourceAttribute"
  ],
  "properties": {
    "sourceAttribute": {
      "title": "Source Attribute",
      "description": "Attribute holding the compressed geometry to apply, as written by Geometry Extractor. The attribute is removed once its geometry has been applied, and a feature that does not carry it passes through unchanged. A feature whose stored geometry cannot be decoded is sent to the rejected port.",
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
* features
### Output Ports
* features
* rejected
### Category
* Geometry

## Geometry Splitter
### Type
* processor
### Description
Splits multi-part geometries into individual single-geometry features.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Geometry Splitter Parameters",
  "description": "Configure how multi-part geometries are split into individual features.",
  "type": "object",
  "properties": {
    "groupBy": {
      "title": "Group By",
      "description": "Attribute key to group split members by. Members sharing the same value for this attribute are kept together in a single output feature instead of being split into separate ones; members lacking the attribute are still emitted individually.",
      "default": null,
      "type": [
        "string",
        "null"
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Geometry

## Geometry Validator
### Type
* processor
### Description
Validates feature geometry for issues such as duplicate points, corrupt geometry, or self-intersection.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Geometry Validator Parameters",
  "description": "Configure which validation checks to perform on feature geometries.",
  "type": "object",
  "properties": {
    "disabledOptionalChecks": {
      "title": "Disabled Optional Checks",
      "description": "Advisory checks to disable. Disabled checks do not run and are treated as passing; core validity checks always run. Empty by default, so every optional check runs.",
      "default": [],
      "type": "array",
      "items": {
        "$ref": "#/definitions/OptionalCheck"
      }
    },
    "planarityThreshold": {
      "title": "Planarity Threshold",
      "description": "Optional override for how the planarity check bounds a face's out-of-plane deviation: a scale-invariant `ratio` (the default), or an absolute `maxHeight` in the frame's linear unit (linear-unit frames only).",
      "default": null,
      "anyOf": [
        {
          "$ref": "#/definitions/PlanarityThreshold"
        },
        {
          "type": "null"
        }
      ]
    },
    "duplicateTolerance": {
      "title": "Duplicate Point Tolerance",
      "description": "Optional distance within which two coordinates count as duplicates for the duplicate-points check. Omitted (the default) means exact-equality detection.",
      "default": null,
      "type": [
        "number",
        "null"
      ],
      "format": "double"
    },
    "degenerateThresholds": {
      "title": "Degeneracy Thresholds",
      "description": "Minimum length / area / volume below which the degeneracy check flags a geometry, per dimension. Each defaults to zero, flagging only an exactly-zero measure. Values are in the coordinate unit (the frame's linear unit, e.g. metres).",
      "default": {
        "minLength": 0.0,
        "minArea": 0.0,
        "minVolume": 0.0
      },
      "allOf": [
        {
          "$ref": "#/definitions/DegenerateThresholds"
        }
      ]
    }
  },
  "definitions": {
    "OptionalCheck": {
      "description": "An advisory (optional) validation check that can be individually disabled. A disabled check does not run and is treated as passing. Only checks that the geometry crate classifies as optional are listed here; core validity checks always run and cannot be disabled.",
      "oneOf": [
        {
          "title": "Duplicate Points",
          "description": "Detection of coordinates that repeat within the geometry.",
          "type": "string",
          "enum": [
            "duplicatePoints"
          ]
        },
        {
          "title": "Orientable",
          "description": "Detection of surfaces whose faces cannot be given a consistent orientation.",
          "type": "string",
          "enum": [
            "orientable"
          ]
        },
        {
          "title": "Orientation",
          "description": "Detection of rings wound in the wrong direction.",
          "type": "string",
          "enum": [
            "orientation"
          ]
        },
        {
          "title": "Shell Orientation",
          "description": "Detection of solid shells whose faces point the wrong way.",
          "type": "string",
          "enum": [
            "shellOrientation"
          ]
        }
      ]
    },
    "PlanarityThreshold": {
      "description": "How the planarity check bounds a face's out-of-plane deviation.",
      "oneOf": [
        {
          "title": "Ratio",
          "description": "Dimensionless ratio of the face's convex-hull minimum height to its diameter; scale-invariant.",
          "type": "object",
          "required": [
            "ratio"
          ],
          "properties": {
            "ratio": {
              "type": "number",
              "format": "double"
            }
          },
          "additionalProperties": false
        },
        {
          "title": "Max Height",
          "description": "Absolute maximum out-of-plane height, in the coordinate unit (metres). Applied only in a linear-unit frame, where the planarity check runs.",
          "type": "object",
          "required": [
            "maxHeight"
          ],
          "properties": {
            "maxHeight": {
              "type": "number",
              "format": "double"
            }
          },
          "additionalProperties": false
        }
      ]
    },
    "DegenerateThresholds": {
      "description": "The smallest measure a geometry may have before the degeneracy check flags it, per dimension. Each threshold applies to geometries of its dimension. Values are in the coordinate unit (the frame's linear unit, e.g. metres). Each defaults to zero, flagging only an exactly-zero measure.",
      "type": "object",
      "properties": {
        "minLength": {
          "title": "Minimum Length",
          "description": "Shortest length a 1D geometry (line or ring edge) may have before it is flagged.",
          "default": 0.0,
          "type": "number",
          "format": "double"
        },
        "minArea": {
          "title": "Minimum Area",
          "description": "Smallest area a 2D geometry (face or ring) may have before it is flagged.",
          "default": 0.0,
          "type": "number",
          "format": "double"
        },
        "minVolume": {
          "title": "Minimum Volume",
          "description": "Smallest volume a 3D geometry (solid) may have before it is flagged.",
          "default": 0.0,
          "type": "number",
          "format": "double"
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
* success
* failed
* rejected
### Category
* Geometry

## Geometry Value Filter
### Type
* processor
### Description
Filter Features by Geometry Value Type
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* none
* geometry2d
* geometry3d
* cityGml
### Category
* Geometry

## Grid Divider
### Type
* processor
### Description
Divides polygon geometries into a regular grid of equal-sized cells.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Grid Divider Parameters",
  "description": "Configure the size of the grid cells and how features are grouped onto a shared grid.",
  "type": "object",
  "required": [
    "cellSize"
  ],
  "properties": {
    "cellSize": {
      "title": "Cell Size",
      "description": "Side length of each grid cell, in the same units as the geometry coordinates. Must be greater than zero.",
      "type": "number",
      "format": "double"
    },
    "completeCellsOnly": {
      "title": "Complete Cells Only",
      "description": "Whether to emit only cells that are whole, discarding the partial cells left where the grid meets the edge of a geometry. Defaults to false.",
      "type": [
        "boolean",
        "null"
      ]
    },
    "groupBy": {
      "title": "Group By Attributes",
      "description": "Attributes whose values group features together. Each group is divided on its own grid origin, derived from that group's combined bounds.",
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
* features
### Output Ports
* features
* rejected
### Category
* Geometry

## HTTP Caller
### Type
* processor
### Description
Calls an HTTP or HTTPS endpoint for each feature and stores the response in feature attributes.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "HTTP Caller Parameters",
  "description": "Configure the HTTP request made for each feature and how the response is stored",
  "type": "object",
  "required": [
    "url"
  ],
  "properties": {
    "url": {
      "title": "URL",
      "description": "The URL to request, evaluated for each feature (supports expressions). Only http and https URLs are allowed, and requests to private or internal network addresses are blocked.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "method": {
      "title": "HTTP Method",
      "description": "The HTTP method to use for the request (default: GET)",
      "default": "GET",
      "allOf": [
        {
          "$ref": "#/definitions/HttpMethod"
        }
      ]
    },
    "authentication": {
      "title": "Authentication",
      "description": "Authentication method and credentials for the request",
      "anyOf": [
        {
          "$ref": "#/definitions/Authentication"
        },
        {
          "type": "null"
        }
      ]
    },
    "customHeaders": {
      "title": "Custom Headers",
      "description": "Additional HTTP headers to include in the request",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/HeaderParam"
      }
    },
    "queryParameters": {
      "title": "Query Parameters",
      "description": "URL query parameters to append to the request",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/QueryParam"
      }
    },
    "requestBody": {
      "title": "Request Body",
      "description": "The body content to send with the request",
      "anyOf": [
        {
          "$ref": "#/definitions/RequestBody"
        },
        {
          "type": "null"
        }
      ]
    },
    "response": {
      "title": "Response Configuration",
      "description": "Configure how the response is stored on the feature",
      "anyOf": [
        {
          "$ref": "#/definitions/ResponseConfig"
        },
        {
          "type": "null"
        }
      ]
    },
    "retry": {
      "title": "Retry Configuration",
      "description": "Settings for automatic retry on failures",
      "anyOf": [
        {
          "$ref": "#/definitions/RetryConfig"
        },
        {
          "type": "null"
        }
      ]
    },
    "rateLimit": {
      "title": "Rate Limiting",
      "description": "Rate limiting configuration to control request frequency",
      "anyOf": [
        {
          "$ref": "#/definitions/RateLimitConfig"
        },
        {
          "type": "null"
        }
      ]
    },
    "timeouts": {
      "title": "Timeouts",
      "description": "Connection and transfer timeout settings",
      "anyOf": [
        {
          "$ref": "#/definitions/TimeoutConfig"
        },
        {
          "type": "null"
        }
      ]
    },
    "httpOptions": {
      "title": "HTTP Options",
      "description": "HTTP client behavior settings (SSL verification, redirects, user agent)",
      "anyOf": [
        {
          "$ref": "#/definitions/HttpOptions"
        },
        {
          "type": "null"
        }
      ]
    }
  },
  "definitions": {
    "HttpMethod": {
      "title": "HTTP Method",
      "description": "The HTTP request method to use",
      "oneOf": [
        {
          "title": "GET",
          "description": "Retrieve data from the server",
          "type": "string",
          "enum": [
            "GET"
          ]
        },
        {
          "title": "POST",
          "description": "Submit data to the server",
          "type": "string",
          "enum": [
            "POST"
          ]
        },
        {
          "title": "PUT",
          "description": "Update or create a resource",
          "type": "string",
          "enum": [
            "PUT"
          ]
        },
        {
          "title": "DELETE",
          "description": "Delete a resource",
          "type": "string",
          "enum": [
            "DELETE"
          ]
        },
        {
          "title": "PATCH",
          "description": "Partially update a resource",
          "type": "string",
          "enum": [
            "PATCH"
          ]
        },
        {
          "title": "HEAD",
          "description": "Retrieve headers only (no body)",
          "type": "string",
          "enum": [
            "HEAD"
          ]
        },
        {
          "title": "OPTIONS",
          "description": "Query supported methods",
          "type": "string",
          "enum": [
            "OPTIONS"
          ]
        }
      ]
    },
    "Authentication": {
      "title": "Authentication",
      "description": "Authentication method and credentials for HTTP requests",
      "oneOf": [
        {
          "title": "Basic Authentication",
          "description": "HTTP Basic authentication with username and password",
          "type": "object",
          "required": [
            "password",
            "type",
            "username"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "basic"
              ]
            },
            "username": {
              "title": "Username",
              "description": "The username for basic authentication",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr",
                    "string"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            },
            "password": {
              "title": "Password",
              "description": "The password for basic authentication",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr",
                    "string"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            }
          }
        },
        {
          "title": "Bearer Token",
          "description": "Bearer token authentication (OAuth 2.0)",
          "type": "object",
          "required": [
            "token",
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "bearer"
              ]
            },
            "token": {
              "title": "Token",
              "description": "The bearer token value",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr",
                    "string"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            }
          }
        },
        {
          "title": "API Key",
          "description": "API key authentication in header or query parameter",
          "type": "object",
          "required": [
            "keyName",
            "keyValue",
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "apiKey"
              ]
            },
            "keyName": {
              "title": "Key Name",
              "description": "The name of the API key parameter",
              "type": "string"
            },
            "keyValue": {
              "title": "Key Value",
              "description": "The API key value",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr",
                    "string"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            },
            "location": {
              "title": "Location",
              "description": "Where to include the API key (header or query parameter)",
              "default": "header",
              "allOf": [
                {
                  "$ref": "#/definitions/ApiKeyLocation"
                }
              ]
            }
          }
        }
      ]
    },
    "ApiKeyLocation": {
      "title": "API Key Location",
      "description": "Where to include the API key in the request",
      "oneOf": [
        {
          "title": "Header",
          "description": "Include API key in HTTP header",
          "type": "string",
          "enum": [
            "header"
          ]
        },
        {
          "title": "Query Parameter",
          "description": "Include API key in URL query string",
          "type": "string",
          "enum": [
            "query"
          ]
        }
      ]
    },
    "HeaderParam": {
      "title": "HTTP Header",
      "description": "A custom HTTP header to include in the request",
      "type": "object",
      "required": [
        "name",
        "value"
      ],
      "properties": {
        "name": {
          "title": "Header Name",
          "description": "The name of the HTTP header",
          "type": "string"
        },
        "value": {
          "title": "Header Value",
          "description": "The value of the header (supports expressions)",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr",
                "string"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        }
      }
    },
    "QueryParam": {
      "title": "Query Parameter",
      "description": "A URL query parameter to append to the request",
      "type": "object",
      "required": [
        "name",
        "value"
      ],
      "properties": {
        "name": {
          "title": "Parameter Name",
          "description": "The name of the query parameter",
          "type": "string"
        },
        "value": {
          "title": "Parameter Value",
          "description": "The value of the parameter (supports expressions)",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr",
                "string"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        }
      }
    },
    "RequestBody": {
      "title": "Request Body",
      "description": "The body content to send with the HTTP request",
      "oneOf": [
        {
          "title": "Text Body",
          "description": "Send text or JSON content",
          "type": "object",
          "required": [
            "content",
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "text"
              ]
            },
            "content": {
              "title": "Content",
              "description": "The text content to send (supports expressions)",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr",
                    "string"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            },
            "contentType": {
              "title": "Content Type",
              "description": "Content-Type header for the body, such as application/json or text/plain",
              "type": [
                "string",
                "null"
              ]
            }
          }
        },
        {
          "title": "Binary Body",
          "description": "Send binary data from base64 or file",
          "type": "object",
          "required": [
            "source",
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "binary"
              ]
            },
            "source": {
              "title": "Binary Source",
              "description": "Source of the binary data (base64 string or file path)",
              "allOf": [
                {
                  "$ref": "#/definitions/BinarySource"
                }
              ]
            },
            "contentType": {
              "title": "Content Type",
              "description": "Content-Type header for the body (default: application/octet-stream)",
              "type": [
                "string",
                "null"
              ]
            }
          }
        },
        {
          "title": "Form URL Encoded",
          "description": "Send application/x-www-form-urlencoded data",
          "type": "object",
          "required": [
            "fields",
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "formUrlEncoded"
              ]
            },
            "fields": {
              "title": "Form Fields",
              "description": "List of form field name-value pairs",
              "type": "array",
              "items": {
                "$ref": "#/definitions/FormField"
              }
            }
          }
        },
        {
          "title": "Multipart Form Data",
          "description": "Send multipart/form-data (for file uploads); cannot be combined with retry",
          "type": "object",
          "required": [
            "parts",
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "multipart"
              ]
            },
            "parts": {
              "title": "Parts",
              "description": "List of multipart form parts (text fields or file uploads)",
              "type": "array",
              "items": {
                "$ref": "#/definitions/MultipartPart"
              }
            }
          }
        }
      ]
    },
    "BinarySource": {
      "title": "Binary Source",
      "description": "Source of binary data for request body",
      "oneOf": [
        {
          "title": "Base64 Encoded",
          "description": "Binary data encoded as base64 string",
          "type": "object",
          "required": [
            "data",
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "base64"
              ]
            },
            "data": {
              "title": "Data",
              "description": "Base64-encoded binary data (supports expressions)",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr",
                    "string"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            }
          }
        },
        {
          "title": "From File",
          "description": "Read binary data from a file",
          "type": "object",
          "required": [
            "path",
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "file"
              ]
            },
            "path": {
              "title": "File Path",
              "description": "Path to the file to read (supports expressions)",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr",
                    "string"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            }
          }
        }
      ]
    },
    "FormField": {
      "title": "Form Field",
      "description": "A name-value pair for URL-encoded form data",
      "type": "object",
      "required": [
        "name",
        "value"
      ],
      "properties": {
        "name": {
          "title": "Field Name",
          "description": "The name of the form field",
          "type": "string"
        },
        "value": {
          "title": "Field Value",
          "description": "The value of the form field (supports expressions)",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr",
                "string"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        }
      }
    },
    "MultipartPart": {
      "title": "Multipart Part",
      "description": "A part in a multipart/form-data request",
      "oneOf": [
        {
          "title": "Text Field",
          "description": "A text field in the multipart form",
          "type": "object",
          "required": [
            "name",
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "text"
              ]
            },
            "name": {
              "title": "Field Name",
              "description": "The name of the form field",
              "type": "string"
            },
            "value": {
              "title": "Field Value",
              "description": "The value of the form field (supports expressions)",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr",
                    "string"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            }
          }
        },
        {
          "title": "File Upload",
          "description": "A file upload in the multipart form",
          "type": "object",
          "required": [
            "name",
            "source",
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "file"
              ]
            },
            "name": {
              "title": "Field Name",
              "description": "The name of the file upload field",
              "type": "string"
            },
            "source": {
              "title": "File Source",
              "description": "Source of the file data (base64 or file path)",
              "allOf": [
                {
                  "$ref": "#/definitions/BinarySource"
                }
              ]
            },
            "filename": {
              "title": "Filename",
              "description": "The filename to send in the Content-Disposition header",
              "type": [
                "string",
                "null"
              ]
            },
            "contentType": {
              "title": "Content Type",
              "description": "MIME type of the file",
              "type": [
                "string",
                "null"
              ]
            }
          }
        }
      ]
    },
    "ResponseConfig": {
      "title": "Response Configuration",
      "description": "Configure how the response is stored. The status code, response headers, and any error message are always stored in the `_http_status_code`, `_headers`, and `_http_error` attributes.",
      "type": "object",
      "properties": {
        "responseBodyAttribute": {
          "title": "Response Body Attribute",
          "description": "Feature attribute name to store the response body (default: `_response_body`)",
          "default": "_response_body",
          "type": "string"
        },
        "responseHandling": {
          "title": "Response Handling",
          "description": "Whether to store the response body in a feature attribute or save it to a file (default: attribute)",
          "anyOf": [
            {
              "$ref": "#/definitions/ResponseHandling"
            },
            {
              "type": "null"
            }
          ]
        },
        "responseEncoding": {
          "title": "Response Encoding",
          "description": "How to store the response body: as UTF-8 text or as a base64-encoded string. When omitted, the encoding is chosen from the response's Content-Type header.",
          "anyOf": [
            {
              "$ref": "#/definitions/ResponseEncoding"
            },
            {
              "type": "null"
            }
          ]
        },
        "autoDetectEncoding": {
          "title": "Auto Detect Encoding",
          "description": "Choose text or base64 storage from the response's Content-Type header when Response Encoding is not set (default: true)",
          "type": [
            "boolean",
            "null"
          ]
        },
        "maxResponseSize": {
          "title": "Max Response Size",
          "description": "Maximum response body size in bytes; the download is stopped and the feature rejected when a response exceeds it (unlimited if not set)",
          "type": [
            "integer",
            "null"
          ],
          "format": "uint64",
          "minimum": 0.0
        }
      }
    },
    "ResponseHandling": {
      "title": "Response Handling",
      "description": "How to handle the HTTP response data",
      "oneOf": [
        {
          "title": "Store in Attribute",
          "description": "Store the response body in a feature attribute",
          "type": "object",
          "required": [
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "attribute"
              ]
            }
          }
        },
        {
          "title": "Save to File",
          "description": "Save the response body to a file under the job's output directory, recording its location in the `_response_file_path` attribute",
          "type": "object",
          "required": [
            "path",
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "file"
              ]
            },
            "path": {
              "title": "File Path",
              "description": "Relative path under the job's output directory where the response is saved (supports expressions)",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr",
                    "string"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            }
          }
        }
      ]
    },
    "ResponseEncoding": {
      "title": "Response Encoding",
      "description": "How to store the response body",
      "oneOf": [
        {
          "title": "Text",
          "description": "Store the response body as UTF-8 text",
          "type": "string",
          "enum": [
            "text"
          ]
        },
        {
          "title": "Base64",
          "description": "Store the response body as a base64-encoded string (for binary data)",
          "type": "string",
          "enum": [
            "base64"
          ]
        }
      ]
    },
    "RetryConfig": {
      "title": "Retry Configuration",
      "description": "Configure automatic retry behavior for failed requests",
      "type": "object",
      "properties": {
        "maxAttempts": {
          "title": "Max Attempts",
          "description": "Maximum total number of attempts including the initial request; 1 disables retries (default: 3)",
          "default": 3,
          "type": "integer",
          "format": "uint32",
          "minimum": 0.0
        },
        "initialDelayMs": {
          "title": "Initial Delay",
          "description": "Initial delay in milliseconds before the first retry (default: 100)",
          "default": 100,
          "type": "integer",
          "format": "uint64",
          "minimum": 0.0
        },
        "backoffMultiplier": {
          "title": "Backoff Multiplier",
          "description": "Multiplier for exponential backoff between retries (default: 2.0)",
          "default": 2.0,
          "type": "number",
          "format": "double"
        },
        "maxDelayMs": {
          "title": "Max Delay",
          "description": "Maximum delay in milliseconds between retries, also capping delays requested by the Retry-After header (default: 10000)",
          "default": 10000,
          "type": "integer",
          "format": "uint64",
          "minimum": 0.0
        },
        "retryOnStatus": {
          "title": "Retry on Status Codes",
          "description": "HTTP status codes that trigger a retry, such as [429, 503]. When omitted, all 5xx status codes are retried.",
          "type": [
            "array",
            "null"
          ],
          "items": {
            "type": "integer",
            "format": "uint16",
            "minimum": 0.0
          }
        },
        "honorRetryAfter": {
          "title": "Honor Retry-After Header",
          "description": "Whether to respect the Retry-After response header, in seconds or HTTP-date form, when scheduling a retry (default: true)",
          "default": true,
          "type": "boolean"
        }
      }
    },
    "RateLimitConfig": {
      "title": "Rate Limit Configuration",
      "description": "Control the rate of HTTP requests to avoid overwhelming the server",
      "type": "object",
      "required": [
        "requests"
      ],
      "properties": {
        "requests": {
          "title": "Requests",
          "description": "Maximum number of requests allowed within the interval",
          "type": "integer",
          "format": "uint32",
          "minimum": 0.0
        },
        "intervalMs": {
          "title": "Interval",
          "description": "Time interval in milliseconds for the rate limit (default: 1000)",
          "default": 1000,
          "type": "integer",
          "format": "uint64",
          "minimum": 0.0
        },
        "timing": {
          "title": "Timing Strategy",
          "description": "How to distribute requests within the interval (default: burst)",
          "default": "burst",
          "allOf": [
            {
              "$ref": "#/definitions/TimingStrategy"
            }
          ]
        }
      }
    },
    "TimingStrategy": {
      "title": "Timing Strategy",
      "description": "How to distribute requests within the rate limit interval",
      "oneOf": [
        {
          "title": "Burst",
          "description": "Allow all requests immediately, then pause until next interval",
          "type": "string",
          "enum": [
            "burst"
          ]
        },
        {
          "title": "Distributed",
          "description": "Evenly distribute requests throughout the interval",
          "type": "string",
          "enum": [
            "distributed"
          ]
        }
      ]
    },
    "TimeoutConfig": {
      "title": "Timeout Configuration",
      "description": "Configure connection and transfer timeouts for HTTP requests",
      "type": "object",
      "properties": {
        "connectionTimeout": {
          "title": "Connection Timeout",
          "description": "Maximum time in seconds to establish a connection (default: 60)",
          "type": [
            "integer",
            "null"
          ],
          "format": "uint64",
          "minimum": 0.0
        },
        "transferTimeout": {
          "title": "Transfer Timeout",
          "description": "Maximum time in seconds to complete the entire request (default: 90)",
          "type": [
            "integer",
            "null"
          ],
          "format": "uint64",
          "minimum": 0.0
        }
      }
    },
    "HttpOptions": {
      "title": "HTTP Options",
      "description": "Configure HTTP client behavior",
      "type": "object",
      "properties": {
        "userAgent": {
          "title": "User Agent",
          "description": "Custom User-Agent header value sent with each request",
          "type": [
            "string",
            "null"
          ]
        },
        "verifySsl": {
          "title": "Verify SSL",
          "description": "Whether to verify SSL/TLS certificates; disable only for servers with self-signed certificates (default: true)",
          "type": [
            "boolean",
            "null"
          ]
        },
        "followRedirects": {
          "title": "Follow Redirects",
          "description": "Whether to automatically follow HTTP redirects (default: true)",
          "type": [
            "boolean",
            "null"
          ]
        },
        "maxRedirects": {
          "title": "Max Redirects",
          "description": "Maximum number of redirects to follow (default: 10)",
          "type": [
            "integer",
            "null"
          ],
          "format": "uint8",
          "minimum": 0.0
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* rejected
### Category
* Feature

## Hole Counter
### Type
* processor
### Description
Counts the holes in every face of a feature's geometry and stores the total in an attribute. A geometry that cannot carry a hole, and a feature with no geometry, both count as zero.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Hole Counter Parameters",
  "description": "Where the total number of holes is stored on each feature.",
  "type": "object",
  "required": [
    "outputAttribute"
  ],
  "properties": {
    "outputAttribute": {
      "title": "Output Attribute",
      "description": "Attribute the count is written to, as a number. It is set on every feature, so a geometry with no holes records zero.",
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
* features
### Output Ports
* features
### Category
* Geometry

## Hole Extractor
### Type
* processor
### Description
Splits each face of a geometry into its rings, emitting the exterior ring and every interior ring (hole) as a feature of its own.
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* exterior
* hole
* rejected
### Category
* Geometry

## Horizontal Reprojector
### Type
* processor
### Description
Reprojects feature geometry from one horizontal coordinate system to another using EPSG codes.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Horizontal Reprojector Parameters",
  "description": "Configure the source and target coordinate systems for geometry reprojection.",
  "type": "object",
  "required": [
    "targetEpsgCode"
  ],
  "properties": {
    "targetEpsgCode": {
      "title": "Target EPSG Code",
      "description": "EPSG code to reproject into, as a constant such as \"4326\" or an expression referencing feature attributes.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "sourceEpsgCode": {
      "title": "Source EPSG Code",
      "description": "EPSG code to reproject from, as a constant or an expression. Defaults to the EPSG code carried on the geometry, so setting it is only necessary when the geometry has none or carries the wrong one.",
      "default": null,
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Geometry

## Image Rasterizer
### Type
* processor
### Description
Converts vector geometries to a raster image using configurable overlap resolution.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Image Rasterizer Parameters",
  "description": "Configure the size of the rendered image, where it is written, and how overlapping geometries are resolved.",
  "type": "object",
  "properties": {
    "imageWidth": {
      "title": "Image Width",
      "description": "Width of the output image in pixels. The height follows from the extent of the input geometries, preserving their aspect ratio.",
      "default": 1000,
      "type": "integer",
      "format": "uint32",
      "minimum": 0.0
    },
    "saveTo": {
      "title": "Save To",
      "description": "Path to write the generated image to. When omitted, the image is written to the cache directory.",
      "default": null,
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "onOverlap": {
      "title": "On Overlap",
      "description": "How to colour a pixel covered by more than one geometry. When omitted, overlapping geometries are drawn in the order they arrive.",
      "default": null,
      "anyOf": [
        {
          "$ref": "#/definitions/OnOverlap"
        },
        {
          "type": "null"
        }
      ]
    }
  },
  "definitions": {
    "OnOverlap": {
      "description": "Overlap resolution strategy for rasterized pixels",
      "oneOf": [
        {
          "title": "Take Last",
          "description": "Keeps the colour of the last polygon drawn over the pixel.",
          "type": "string",
          "enum": [
            "takeLast"
          ]
        },
        {
          "title": "Take First",
          "description": "Keeps the colour of the first polygon drawn over the pixel.",
          "type": "string",
          "enum": [
            "takeFirst"
          ]
        },
        {
          "title": "Maximum",
          "description": "Keeps the colour of the overlapping polygon whose expression evaluates highest.",
          "type": "object",
          "required": [
            "max"
          ],
          "properties": {
            "max": {
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            }
          },
          "additionalProperties": false
        },
        {
          "title": "Minimum",
          "description": "Keeps the colour of the overlapping polygon whose expression evaluates lowest.",
          "type": "object",
          "required": [
            "min"
          ],
          "properties": {
            "min": {
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            }
          },
          "additionalProperties": false
        },
        {
          "title": "Sum",
          "description": "Adds the RGB channels of every overlapping polygon, saturating at full intensity.",
          "type": "string",
          "enum": [
            "sum"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* features
* texture-coordinates
### Output Ports
* features
* textured
* texture-bounds
* rejected
### Category
* Geometry

## Input Router
### Type
* processor
### Description
Forwards features from the parent workflow into a sub-workflow.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Input Router Parameters",
  "description": "Configuration for receiving features from the parent workflow.",
  "type": "object",
  "required": [
    "routingPort"
  ],
  "properties": {
    "routingPort": {
      "title": "Routing Port",
      "description": "Name of the parent workflow port whose features enter the sub-workflow through this router.",
      "type": "string"
    }
  }
}
```
### Input Ports
### Output Ports
* features
### Category
* Filter

## JP Standard Grid Accumulator
### Type
* processor
### Description
Divides geometries into Japanese standard mesh grid (1km) and adds mesh codes to features
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* features
* rejected
### Category
* Geometry

## JSON Fragmenter
### Type
* processor
### Description
Fragments JSON documents into individual features based on a JSONPath query
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "JSON Fragmenter Parameters",
  "description": "Configuration for fragmenting JSON documents into individual features.",
  "oneOf": [
    {
      "description": "Read JSON from a feature attribute",
      "type": "object",
      "required": [
        "inputSource",
        "jsonAttribute",
        "jsonQuery"
      ],
      "properties": {
        "inputSource": {
          "type": "string",
          "enum": [
            "attribute"
          ]
        },
        "jsonAttribute": {
          "description": "The attribute containing the JSON text to fragment",
          "allOf": [
            {
              "$ref": "#/definitions/Attribute"
            }
          ]
        },
        "jsonQuery": {
          "description": "JSONPath expression to select elements (e.g., \"$[*]\", \"$.results[*]\")",
          "type": "string"
        },
        "flattenQueryResult": {
          "description": "If true, flatten JSON object keys into feature attributes",
          "default": false,
          "type": "boolean"
        },
        "recursivelyFlatten": {
          "description": "If true, recursively flatten nested objects using dot-separated keys",
          "default": false,
          "type": "boolean"
        },
        "attributePrefix": {
          "description": "Optional prefix for flattened attribute names",
          "default": null,
          "type": [
            "string",
            "null"
          ]
        },
        "rejectNoFragments": {
          "description": "If true, reject features that produce no fragments",
          "default": false,
          "type": "boolean"
        }
      }
    },
    {
      "description": "Read JSON from a file path or URL",
      "type": "object",
      "required": [
        "inputSource",
        "jsonQuery",
        "path"
      ],
      "properties": {
        "inputSource": {
          "type": "string",
          "enum": [
            "fileUrl"
          ]
        },
        "path": {
          "description": "Expression evaluating to the file path or URL containing JSON",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr",
                "string"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "jsonQuery": {
          "description": "JSONPath expression to select elements (e.g., \"$[*]\", \"$.results[*]\")",
          "type": "string"
        },
        "flattenQueryResult": {
          "description": "If true, flatten JSON object keys into feature attributes",
          "default": false,
          "type": "boolean"
        },
        "recursivelyFlatten": {
          "description": "If true, recursively flatten nested objects using dot-separated keys",
          "default": false,
          "type": "boolean"
        },
        "attributePrefix": {
          "description": "Optional prefix for flattened attribute names",
          "default": null,
          "type": [
            "string",
            "null"
          ]
        },
        "rejectNoFragments": {
          "description": "If true, reject features that produce no fragments",
          "default": false,
          "type": "boolean"
        }
      }
    }
  ],
  "definitions": {
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* rejected
### Category
* Feature

## JSON Reader
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
      "description": "Expression that returns the path to the input file, either a literal path or a variable reference.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
### Output Ports
* features
### Category
* Input

## JSON Writer
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
    "output": {
      "title": "Output File",
      "description": "Output path or expression for the JSON file to create.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "converter": {
      "title": "Converter Expression",
      "description": "Expression that transforms features into the JSON value to write. When omitted, features are written as an array of their attributes.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
### Category
* Output

## Line On Line Overlayer
### Type
* processor
### Description
Splits lines where they cross, recording on each resulting segment how many input lines run along it, and emits every crossing as a point feature. Inputs must be flat 2D geometries sharing one coordinate frame; place a Two Dimension Forcer or a Coordinate Frame Reprojector upstream to flatten or unify them.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Line On Line Overlayer Parameters",
  "description": "Sets which lines are crossed against each other, how far apart two vertices may be and still count as one, and what the resulting segments record about the lines they came from.",
  "type": "object",
  "required": [
    "tolerance"
  ],
  "properties": {
    "tolerance": {
      "title": "Tolerance",
      "description": "Distance below which two vertices are treated as the same point, in the unit of the input's coordinate frame. It decides which crossings split a line, which crossings are the same crossing, and which segments coincide; segments shorter than it are dropped. Must be greater than zero, or no line is ever split.",
      "type": "number",
      "format": "double"
    },
    "groupBy": {
      "title": "Group By Attributes",
      "description": "Attributes whose values decide which lines are crossed against each other — only lines matching on all of them are compared. When omitted, every line is crossed against every other.",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "outputAttribute": {
      "title": "Overlap Count Attribute",
      "description": "Attribute that receives the number of input lines running along the resulting segment. Defaults to `overlayCount`.",
      "default": "overlayCount",
      "type": "string"
    },
    "listAttribute": {
      "title": "List Attribute",
      "description": "Attribute that receives one entry per line running along the resulting segment, each holding that line's own attributes. When omitted, no list is written.",
      "type": [
        "string",
        "null"
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
* features
### Output Ports
* point
* line
* rejected
### Category
* Geometry

## List Concatenator
### Type
* processor
### Description
Joins one attribute's value from every element of a list attribute into a single string. Elements that are not key-value pairs, or that lack the attribute, are skipped.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "List Concatenator Parameters",
  "description": "Which list to read, which attribute to take from its elements, and where the result goes.",
  "type": "object",
  "required": [
    "elementAttribute",
    "list",
    "outputAttribute"
  ],
  "properties": {
    "list": {
      "title": "List",
      "description": "Attribute holding the list to read. A feature whose attribute is missing, or is not a list, passes through unchanged.",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "elementAttribute": {
      "title": "Element Attribute",
      "description": "Attribute to take from each element of the list. An element that is not a set of key-value pairs, or that does not carry this attribute, contributes nothing.",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "outputAttribute": {
      "title": "Output Attribute",
      "description": "Attribute the joined string is written to.",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "separator": {
      "title": "Separator",
      "description": "Text placed between consecutive values. Defaults to a comma.",
      "default": ",",
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
* features
### Output Ports
* features
### Category
* Attribute

## List Exploder
### Type
* processor
### Description
Creates one feature per element of a list attribute, merging the element's key-value pairs into the feature's attributes and removing the source attribute. A feature whose attribute is missing, empty, or not a list of key-value pairs produces nothing and is rejected instead.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "List Exploder Parameters",
  "description": "Configures which list attribute is expanded into individual features.",
  "type": "object",
  "required": [
    "sourceAttribute"
  ],
  "properties": {
    "sourceAttribute": {
      "title": "Source Attribute",
      "description": "Attribute holding a list of key-value pairs, one entry per feature to create.",
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
* features
### Output Ports
* features
* rejected
### Category
* Transform

## List Indexer
### Type
* processor
### Description
Copies the key-value pairs of one element of a list attribute onto the feature itself, overwriting an attribute of the same name and leaving the rest in place.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "List Indexer Parameters",
  "description": "Which list to read, which of its elements to copy, and how to name what is copied.",
  "type": "object",
  "required": [
    "index",
    "list"
  ],
  "properties": {
    "list": {
      "title": "List",
      "description": "Attribute holding the list to read. A feature whose attribute is missing, or is not a list, passes through unchanged.",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "index": {
      "title": "Index",
      "description": "Position of the element to copy, counting from zero. A feature whose list is shorter than this, or whose element at this position is not a set of key-value pairs, passes through unchanged.",
      "type": "integer",
      "format": "uint",
      "minimum": 0.0
    },
    "copiedAttributePrefix": {
      "title": "Copied Attribute Prefix",
      "description": "Text placed before each copied attribute name. Omitted by default.",
      "default": null,
      "type": [
        "string",
        "null"
      ]
    },
    "copiedAttributeSuffix": {
      "title": "Copied Attribute Suffix",
      "description": "Text placed after each copied attribute name. Omitted by default.",
      "default": null,
      "type": [
        "string",
        "null"
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
* features
### Output Ports
* features
### Category
* Attribute

## MVT Writer
### Type
* sink
### Description
Writes features to Mapbox Vector Tiles (MVT) format.
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
    "output": {
      "title": "Output Directory",
      "description": "Output directory path or expression where the generated MVT tiles are written.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "layerName": {
      "title": "Layer Name",
      "description": "Name or expression for the layer within the generated tiles.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "minZoom": {
      "title": "Minimum Zoom",
      "description": "Lowest zoom level to generate tiles for.",
      "type": "integer",
      "format": "uint8",
      "minimum": 0.0
    },
    "maxZoom": {
      "title": "Maximum Zoom",
      "description": "Highest zoom level to generate tiles for.",
      "type": "integer",
      "format": "uint8",
      "minimum": 0.0
    },
    "compressOutput": {
      "title": "Compressed Output Path",
      "description": "Optional path where a compressed archive of the tiles is also written.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "schemaKey": {
      "title": "Schema Key",
      "description": "Attribute key used to match data and schema features for attribute filtering and casting. This attribute is excluded from output.",
      "type": [
        "string",
        "null"
      ]
    },
    "skipUnexposedAttributes": {
      "title": "Skip Unexposed Attributes",
      "description": "Whether to skip attributes whose keys begin with a double underscore.",
      "type": [
        "boolean",
        "null"
      ]
    },
    "colonToUnderscore": {
      "title": "Colon to Underscore",
      "description": "Whether to replace colons in attribute keys (such as those from XML namespaces) with underscores.",
      "type": [
        "boolean",
        "null"
      ]
    },
    "extent": {
      "title": "Tile Extent",
      "description": "Coordinate grid resolution within each tile. Higher values preserve more positional precision for high-detail data at the cost of larger tiles. Defaults to 4096, the MVT standard.",
      "type": [
        "integer",
        "null"
      ],
      "format": "uint32",
      "minimum": 0.0
    },
    "maxTileBytes": {
      "title": "Maximum Tile Size",
      "description": "Target maximum encoded size per tile, in bytes. When exceeded, the least visually significant features are dropped until the tile fits. Defaults to 500,000.",
      "default": 500000,
      "type": "integer",
      "format": "uint64",
      "minimum": 0.0
    },
    "arrayMapSeparator": {
      "title": "Array/Map Separator",
      "description": "Separator joining a nested array or map attribute to its child key or index when flattening it into tags. Leave unset to drop array and map attributes from the output entirely.",
      "type": [
        "string",
        "null"
      ]
    }
  }
}
```
### Input Ports
* features
* schema
### Output Ports
### Category
* Output

## Neighbor Finder
### Type
* processor
### Description
Finds the closest candidate features for each base feature based on spatial proximity
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Neighbor Finder Parameters",
  "description": "Configuration for finding spatial neighbors between base and candidate features.",
  "type": "object",
  "properties": {
    "numClosest": {
      "description": "Number of closest neighbors to find per base feature. Must be >= 1.",
      "default": 1,
      "type": "integer",
      "format": "uint",
      "minimum": 1.0
    },
    "maxDistance": {
      "description": "Maximum distance threshold for matching. If None, no distance limit is applied. Units depend on the distanceMetric: native units for Euclidean, meters for Haversine.",
      "type": [
        "number",
        "null"
      ],
      "format": "double"
    },
    "distanceAttribute": {
      "description": "Name of the attribute to store the computed distance to the nearest neighbor.",
      "default": "_neighbor_distance",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "neighborIndexAttribute": {
      "description": "Name of the attribute to store the neighbor index (0-based rank) when num_closest > 1 and merge_strategy is \"repeatBase\". Set to empty string to suppress.",
      "default": "_neighbor_index",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "attributePrefix": {
      "description": "Prefix applied to transferred candidate attributes to avoid collisions.",
      "default": "_neighbor_",
      "type": "string"
    },
    "attributesToTransfer": {
      "description": "List of candidate attributes to transfer. Empty list means all attributes are transferred.",
      "default": [],
      "type": "array",
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "mergeStrategy": {
      "description": "Controls how multiple neighbors are represented on the output.",
      "default": "closest",
      "allOf": [
        {
          "$ref": "#/definitions/MergeStrategy"
        }
      ]
    },
    "distanceMetric": {
      "description": "Method used to compute distance between two features.",
      "default": "euclidean2d",
      "allOf": [
        {
          "$ref": "#/definitions/DistanceMetric"
        }
      ]
    }
  },
  "definitions": {
    "Attribute": {
      "type": "string"
    },
    "MergeStrategy": {
      "description": "Merge strategy for handling multiple neighbors.",
      "oneOf": [
        {
          "description": "Only the single closest neighbor is merged onto the Base feature.",
          "type": "string",
          "enum": [
            "closest"
          ]
        },
        {
          "description": "One output feature is emitted per neighbor. Base attributes are duplicated across all rows, mirroring FME's default multi-neighbor output.",
          "type": "string",
          "enum": [
            "repeatBase"
          ]
        },
        {
          "description": "A single Base feature is emitted; each transferred attribute becomes an ordered array sorted by ascending distance.",
          "type": "string",
          "enum": [
            "arrayAttributes"
          ]
        }
      ]
    },
    "DistanceMetric": {
      "description": "Distance metric for computing spatial proximity.",
      "oneOf": [
        {
          "description": "2D Euclidean distance using X and Y coordinates. Z is ignored.",
          "type": "string",
          "enum": [
            "euclidean2d"
          ]
        },
        {
          "description": "3D Euclidean distance using X, Y, and Z coordinates.",
          "type": "string",
          "enum": [
            "euclidean3d"
          ]
        },
        {
          "description": "Great-circle distance treating X as longitude (degrees) and Y as latitude (degrees). Output is in meters. Intended for WGS-84 inputs.\n\nNote: Distance is computed between representative points (centroids). For accurate results, input features should be relatively small (e.g., buildings, local roads). Large geometries (e.g., countries, large water bodies) may produce inaccurate distances because their centroids may not represent their spatial extent well.",
          "type": "string",
          "enum": [
            "haversine"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* base
* candidate
### Output Ports
* matched
* unmatched
* rejected
### Category
* Geometry

## Noop Processor
### Type
* processor
### Description
Passes features through unchanged.
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* features
### Category
* Debug

## Noop Sink
### Type
* sink
### Description
Discards all incoming features.
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
### Category
* Debug

## Null Attribute Mapper
### Type
* processor
### Description
Replaces null-like attribute values with configured replacement values, optionally removing or routing affected features.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Null Attribute Mapper Parameters",
  "description": "Detects null-like attribute values and replaces them, removes them, or routes affected features according to per-attribute rules.",
  "type": "object",
  "properties": {
    "scope": {
      "title": "Scope",
      "description": "Which attributes to inspect: only those named in the mappings, or every attribute on the feature.",
      "default": "listed",
      "allOf": [
        {
          "$ref": "#/definitions/Scope"
        }
      ]
    },
    "mappings": {
      "title": "Attribute Mappings",
      "description": "Per-attribute rules describing how each null-like attribute is replaced, removed, or created.",
      "default": [],
      "type": "array",
      "items": {
        "$ref": "#/definitions/AttributeMapping"
      }
    },
    "defaultReplacement": {
      "title": "Default Replacement",
      "description": "What to write for null-like attributes that have no entry in the mappings. Applies only when the scope inspects all attributes. When omitted, those attributes are left unchanged.",
      "default": null,
      "anyOf": [
        {
          "$ref": "#/definitions/NullReplacement"
        },
        {
          "type": "null"
        }
      ]
    },
    "nullDefinition": {
      "title": "Null Definition",
      "description": "States treated as null-like: an explicit null value, a missing attribute, or an empty string. Defaults to null values and missing attributes.",
      "default": [
        "null",
        "missing"
      ],
      "type": "array",
      "items": {
        "$ref": "#/definitions/NullKind"
      }
    },
    "routeNullFeatures": {
      "title": "Route Null Features",
      "description": "When enabled, a copy of each feature that had at least one null-like value is emitted unchanged to a separate output for inspection.",
      "default": false,
      "type": "boolean"
    }
  },
  "definitions": {
    "Scope": {
      "title": "Scope",
      "description": "Which attributes the action inspects.",
      "oneOf": [
        {
          "title": "Listed Attributes",
          "description": "Inspects only the attributes named in the mappings list.",
          "type": "string",
          "enum": [
            "listed"
          ]
        },
        {
          "title": "All Attributes",
          "description": "Inspects every attribute on the feature.",
          "type": "string",
          "enum": [
            "all"
          ]
        }
      ]
    },
    "AttributeMapping": {
      "description": "Per-attribute replacement mapping",
      "type": "object",
      "required": [
        "attribute",
        "replacement"
      ],
      "properties": {
        "attribute": {
          "title": "Attribute",
          "description": "Name of the attribute to inspect.",
          "type": "string"
        },
        "replacement": {
          "title": "Replacement",
          "description": "What to write when the attribute is null-like. Required, so that removing an attribute is stated rather than implied by leaving this out.",
          "allOf": [
            {
              "$ref": "#/definitions/NullReplacement"
            }
          ]
        },
        "onMissing": {
          "title": "On Missing",
          "description": "Behavior when the attribute is absent and not treated as null-like by the null definition.",
          "default": "skip",
          "allOf": [
            {
              "$ref": "#/definitions/OnMissing"
            }
          ]
        }
      }
    },
    "NullReplacement": {
      "title": "Null Replacement",
      "description": "What to write in place of a null-like attribute, and the type to write it as.",
      "oneOf": [
        {
          "title": "Text",
          "description": "Written as text.",
          "type": "object",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "text"
              ]
            },
            "value": {
              "title": "Value",
              "description": "The text to write.",
              "type": "string"
            }
          }
        },
        {
          "title": "Number",
          "description": "Written as a number.",
          "type": "object",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "number"
              ]
            },
            "value": {
              "title": "Value",
              "description": "The number to write.",
              "type": "number"
            }
          }
        },
        {
          "title": "True or False",
          "description": "Written as a true/false value.",
          "type": "object",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "boolean"
              ]
            },
            "value": {
              "title": "Value",
              "description": "The value to write.",
              "type": "boolean"
            }
          }
        },
        {
          "title": "Remove",
          "description": "Removes the attribute from the feature instead of writing a value.",
          "type": "object",
          "required": [
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "remove"
              ]
            }
          }
        }
      ]
    },
    "OnMissing": {
      "title": "On Missing",
      "description": "Behavior when an inspected attribute is absent from the feature.",
      "oneOf": [
        {
          "title": "Skip",
          "description": "Leaves the feature unchanged.",
          "type": "string",
          "enum": [
            "skip"
          ]
        },
        {
          "title": "Create",
          "description": "Adds the attribute using the mapping's replacement value.",
          "type": "string",
          "enum": [
            "create"
          ]
        }
      ]
    },
    "NullKind": {
      "title": "Null Definition",
      "description": "States that are treated as null-like.",
      "oneOf": [
        {
          "title": "Null Value",
          "description": "An attribute present on the feature but holding an explicit null value.",
          "type": "string",
          "enum": [
            "null"
          ]
        },
        {
          "title": "Missing Attribute",
          "description": "An attribute that is absent from the feature.",
          "type": "string",
          "enum": [
            "missing"
          ]
        },
        {
          "title": "Empty String",
          "description": "An attribute holding a text value with no characters.",
          "type": "string",
          "enum": [
            "emptyString"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* has-null
* rejected
### Category
* Attribute

## OBJ Reader
### Type
* source
### Description
Reads 3D models from Wavefront OBJ files, including vertices, faces, normals, texture coordinates, and materials.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "OBJ Reader Parameters",
  "description": "Configuration for reading Wavefront OBJ 3D model files with support for vertices, faces, normals, texture coordinates, and material definitions.",
  "type": "object",
  "properties": {
    "parseMaterials": {
      "title": "Parse Materials",
      "description": "Parses material definitions from MTL files referenced in the OBJ file.",
      "default": true,
      "type": "boolean"
    },
    "materialFile": {
      "title": "Material File",
      "description": "Expression that returns the path to an external MTL file to use instead of mtllib directives in the OBJ file. When specified, this overrides any material library references in the OBJ file.",
      "default": null,
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "triangulate": {
      "title": "Triangulate",
      "description": "Converts polygons with more than 3 vertices into triangles using fan triangulation.",
      "default": false,
      "type": "boolean"
    },
    "mergeGroups": {
      "title": "Merge Groups",
      "description": "Merges all groups and objects into a single feature instead of creating separate features per group or object.",
      "default": false,
      "type": "boolean"
    },
    "includeTexcoords": {
      "title": "Include Texture Coordinates",
      "description": "Includes texture coordinate (UV) data in the output geometry.",
      "default": true,
      "type": "boolean"
    },
    "dataset": {
      "title": "File Path",
      "description": "Expression that returns the path to the input file, either a literal path or a variable reference.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
### Output Ports
* features
### Category
* Input

## OBJ Writer
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
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
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
  }
}
```
### Input Ports
* features
### Output Ports
### Category
* File
* 3D

## Offsetter
### Type
* processor
### Description
Shifts every geometry coordinate by a fixed amount along each axis.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Offsetter Parameters",
  "description": "Amounts added to every geometry coordinate, one per axis, in the coordinate unit of the geometry's frame.",
  "type": "object",
  "properties": {
    "offsetX": {
      "title": "X Offset",
      "description": "Amount added to every X coordinate, in the coordinate unit of the geometry's frame (degrees for a geographic CRS). Defaults to zero.",
      "type": [
        "number",
        "null"
      ],
      "format": "double"
    },
    "offsetY": {
      "title": "Y Offset",
      "description": "Amount added to every Y coordinate, in the coordinate unit of the geometry's frame (degrees for a geographic CRS). Defaults to zero.",
      "type": [
        "number",
        "null"
      ],
      "format": "double"
    },
    "offsetZ": {
      "title": "Z Offset",
      "description": "Amount added to every Z coordinate, in the coordinate unit of the geometry's frame (metres for a geographic CRS). Defaults to zero.",
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
* features
### Output Ports
* features
### Category
* Geometry

## Orientation Extractor
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
* features
### Output Ports
* features
### Category
* Geometry

## Output Router
### Type
* processor
### Description
Forwards features from a sub-workflow back to the parent workflow.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Output Router Parameters",
  "description": "Configuration for returning features to the parent workflow.",
  "type": "object",
  "required": [
    "routingPort"
  ],
  "properties": {
    "routingPort": {
      "title": "Routing Port",
      "description": "Name of the output port under which the sub-workflow's features are exposed to the parent workflow.",
      "type": "string"
    }
  }
}
```
### Input Ports
* features
### Output Ports
### Category
* Filter

## PLATEAU3.AttributeFlattener
### Type
* processor
### Description
Flattens hierarchical PLATEAU3 building attributes into flat structure for analysis
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* features
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
* features
### Output Ports
* features
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
* features
### Output Ports
* lBldgError
* codeError
* features
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
* features
### Output Ports
* features
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
* features
### Output Ports
* features
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
* features
### Output Ports
* features
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
* features
### Output Ports
* features
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
* features
### Output Ports
* features
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
* features
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
    "cityCode": {
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
    },
    "addNsprefixToFeatureTypes": {
      "type": [
        "boolean",
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
    }
  }
}
```
### Input Ports
* features
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
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AttributeFlattener Parameters",
  "type": "object",
  "properties": {
    "existingFlattenAttributes": {
      "description": "When true, only include attributes that were actually used during processing in the schema output. When false (default), include all defined attributes in the schema regardless of usage.",
      "default": false,
      "type": "boolean"
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
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
* features
### Output Ports
* features
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
    "partIdAttribute": {
      "title": "Part ID Attribute",
      "description": "Attribute containing the BuildingPart ID (default: \"featureId\")",
      "default": "featureId",
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
    "fileIndexAttribute": {
      "title": "File Index Attribute",
      "description": "Attribute containing the file index (default: \"fileIndex\")",
      "default": "fileIndex",
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
* features
### Output Ports
* features
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
  "required": [
    "codelistsPath"
  ],
  "properties": {
    "codelistsPath": {
      "description": "Expression evaluating to the PLATEAU codelists directory path.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
* l0405BldgError
* cityCodeError
* features
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
* features
### Output Ports
* features
### Category
* PLATEAU

## PLATEAU4.CityGmlMeshBuilder
### Type
* processor
### Description
Validates CityGML mesh triangles by parsing raw XML: (1) each triangle has exactly 4 vertices, (2) each triangle is closed (first vertex equals last vertex)
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CityGML Mesh Builder Parameters",
  "description": "Configure validation rules for CityGML mesh triangles",
  "type": "object",
  "required": [
    "epsgCode"
  ],
  "properties": {
    "errorAttribute": {
      "title": "Error Attribute Name",
      "description": "Attribute name to store validation error messages (default: \"_validation_error\")",
      "default": "_validation_error",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "epsgCode": {
      "title": "Target EPSG Code",
      "description": "EPSG code for coordinate transformation from source EPSG 6697. Accepts integer or string expression.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
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
* features
### Output Ports
* features
* not_closed
* incorrect_vertices
* wrong_orientation
* degenerate_triangle
* summary
* rejected
### Category
* PLATEAU
* Geometry

## PLATEAU4.CompositeSurfaceContinuityFilter
### Type
* processor
### Description
Checks if a CompositeSurface is continuous (all parts share edges)
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* passed
* failed
* rejected
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
    },
    "epsgCode": {
      "title": "EPSG Code",
      "description": "Japanese Plane Rectangular Coordinate System EPSG code for area calculation",
      "default": {
        "type": "flowExpr",
        "value": "6691"
      },
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* rejected
### Category
* PLATEAU

## PLATEAU4.DomainOfDefinitionValidator
### Type
* processor
### Description
Validates domain of definition of CityGML features
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DomainOfDefinitionValidator Parameters",
  "description": "Configuration for validating domain of definition of CityGML features.",
  "type": "object",
  "properties": {
    "codelistsPath": {
      "description": "Fallback codelists directory path expression. When codelists files are not found at the location relative to the GML file, this path will be used as the base directory for resolving codeSpace references.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* rejected
* duplicateGmlIdStats
### Category
* PLATEAU

## PLATEAU4.FaceExtractor
### Type
* processor
### Description
Validates individual surfaces of WaterBody features for TIN mesh quality
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FaceExtractor Parameters",
  "description": "Configuration for validating individual surfaces of WaterBody features. Always checks vertex count, closure, and orientation of polygons in TIN meshes.",
  "type": "object",
  "properties": {
    "cityGmlPathAttribute": {
      "description": "Attribute name for city_gml_path (default: \"_gml_path\")",
      "default": "_gml_path",
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
* features
### Output Ports
* error
* summary
* passed
* all
### Category
* PLATEAU

## PLATEAU4.FloodingAreaSurfaceGenerator
### Type
* processor
### Description
Generates TIN-based surfaces from flood area polygons for efficient 3D tile generation. Optionally groups features by attribute before combining and triangulating.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "FloodingAreaSurfaceGenerator Parameters",
  "description": "Configuration for generating TIN surfaces from flood area polygons. This processor converts polygons to triangulated surfaces by: 1. Optionally grouping features by an attribute (e.g., udxDirs) 2. Combining all polygons in each group 3. Sampling points along polygon boundaries at regular intervals 4. Optionally adding interior grid points within polygons 5. Performing Delaunay triangulation to create a TIN surface 6. Filtering triangles to keep only those inside the original polygons",
  "type": "object",
  "properties": {
    "pointSpacing": {
      "description": "Spacing between sampled points in meters (default: 50.0). Points are sampled along polygon boundaries and optionally on an interior grid at this spacing interval.",
      "type": [
        "number",
        "null"
      ],
      "format": "double"
    },
    "sampleInterior": {
      "description": "Enable interior grid sampling (default: false). When enabled, points are added on a regular grid inside the polygon to create a more uniform triangulation. This can be slow for large polygons.",
      "default": null,
      "type": [
        "boolean",
        "null"
      ]
    },
    "groupBy": {
      "description": "Attribute name to group features by before combining and triangulating. Features with the same value for this attribute will be processed together. If not specified, each feature is processed individually.",
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
* features
### Output Ports
* features
### Category
* PLATEAU

## PLATEAU4.GmlNameCodeSpaceValidator
### Type
* processor
### Description
Validates that gml:name elements have codeSpace attributes (coded values)
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "GmlNameCodeSpaceValidator Parameters",
  "description": "Configuration for validating gml:name elements to ensure they have codeSpace attributes.",
  "type": "object",
  "properties": {
    "cityGmlPath": {
      "description": "Expression to get the path to the CityGML file",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* gmlNameErrors
* stats
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
* features
### Output Ports
* features
### Category
* PLATEAU

## PLATEAU4.MissingAttributeDetector
### Type
* processor
### Description
Detect missing attributes in PLATEAU features
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "MissingAttributeDetector Parameters",
  "description": "Configuration for detecting missing attributes in PLATEAU features.",
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
* features
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
  "description": "Configuration for extracting object lists from PLATEAU data.",
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
* features
### Output Ports
* features
### Category
* PLATEAU

## PLATEAU4.SolarCityGmlAttributeInserter
### Type
* processor
### Description
Inserts solar radiation measurement attributes into original CityGML files
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "CityGmlAttributeInserterParam",
  "description": "Configuration for inserting measurement attributes into CityGML files.",
  "type": "object",
  "required": [
    "measurements",
    "outputDir"
  ],
  "properties": {
    "outputDir": {
      "description": "Output directory expression for modified CityGML files",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "gmlIdAttribute": {
      "description": "Attribute name on element features holding gml:id (default: \"gmlId\")",
      "default": null,
      "type": [
        "string",
        "null"
      ]
    },
    "pathAttribute": {
      "description": "Attribute name on path features holding the file path (default: \"path\")",
      "default": null,
      "type": [
        "string",
        "null"
      ]
    },
    "measurements": {
      "description": "Measurement definitions to insert as gen:measureAttribute elements",
      "type": "array",
      "items": {
        "$ref": "#/definitions/MeasurementDef"
      }
    },
    "textureImagePath": {
      "description": "Path to the solar radiation texture PNG (texture insertion skipped if absent)",
      "default": null,
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "sourceEpsg": {
      "description": "The projected CRS EPSG code used for rasterization (needed for UV computation)",
      "default": null,
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  },
  "definitions": {
    "MeasurementDef": {
      "description": "A single measurement attribute definition.",
      "type": "object",
      "required": [
        "attribute",
        "name",
        "uom"
      ],
      "properties": {
        "name": {
          "description": "XML name attribute value (e.g. \"年間予測日射量\")",
          "type": "string"
        },
        "attribute": {
          "description": "Feature attribute key holding the numeric value (e.g. \"totalSolarRadiation\")",
          "type": "string"
        },
        "uom": {
          "description": "Unit of measurement (e.g. \"kWh\")",
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
* path
* element
* textureBounds
### Output Ports
* features
### Category
* PLATEAU

## PLATEAU4.SolarPositionCalculator
### Type
* processor
### Description
Calculates solar position (altitude and azimuth) for geographic features using Spencer's algorithm
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "SolarPositionCalculatorParam",
  "oneOf": [
    {
      "type": "object",
      "required": [
        "sourceEpsg",
        "time",
        "type"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "time"
          ]
        },
        "time": {
          "description": "Time expression evaluating to RFC 3339 format (e.g., \"2025-01-11T00:00:00Z\") or date-only format (e.g., \"2025-01-11\" or \"2025-01-11+09:00\"). When hours, minutes, and seconds are omitted they default to zero.",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr",
                "string"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "sourceEpsg": {
          "description": "Source EPSG code expression (required). Evaluates to int (e.g., 6677 for Japan Plane IX).",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "standardMeridian": {
          "description": "Standard meridian in degrees (optional). If not provided, computed as round(longitude / 15) * 15.",
          "default": null,
          "type": [
            "object",
            "null"
          ],
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "outputType": {
          "description": "Output type: unit normal vector or altitude/azimuth angles",
          "default": "unitNormalVector",
          "allOf": [
            {
              "$ref": "#/definitions/OutputType"
            }
          ]
        },
        "outputBelowHorizon": {
          "description": "Whether to output sun positions below the horizon (altitude < 0). Default: false.",
          "default": false,
          "type": "boolean"
        }
      }
    },
    {
      "type": "object",
      "required": [
        "end",
        "sourceEpsg",
        "start",
        "step",
        "stepUnit",
        "type"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "duration"
          ]
        },
        "start": {
          "description": "Start time expression evaluating to RFC 3339 format (e.g., \"2025-01-11T00:00:00Z\") or date-only format (e.g., \"2025-01-11\" or \"2025-01-11+09:00\"). When hours, minutes, and seconds are omitted they default to zero.",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr",
                "string"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "end": {
          "description": "End time expression evaluating to RFC 3339 format (e.g., \"2025-01-12T00:00:00Z\") or date-only format (e.g., \"2025-01-12\" or \"2025-01-12+09:00\"). When hours, minutes, and seconds are omitted they default to zero.",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr",
                "string"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "step": {
          "description": "Step value expression evaluating to an integer",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "stepUnit": {
          "description": "Unit for the step value",
          "allOf": [
            {
              "$ref": "#/definitions/StepUnit"
            }
          ]
        },
        "sourceEpsg": {
          "description": "Source EPSG code expression (required). Evaluates to int (e.g., 6677 for Japan Plane IX).",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "standardMeridian": {
          "description": "Standard meridian in degrees (optional). If not provided, computed as round(longitude / 15) * 15.",
          "default": null,
          "type": [
            "object",
            "null"
          ],
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "outputType": {
          "description": "Output type: unit normal vector or altitude/azimuth angles",
          "default": "unitNormalVector",
          "allOf": [
            {
              "$ref": "#/definitions/OutputType"
            }
          ]
        },
        "outputBelowHorizon": {
          "description": "Whether to output sun positions below the horizon (altitude < 0). Default: false.",
          "default": false,
          "type": "boolean"
        }
      }
    }
  ],
  "definitions": {
    "OutputType": {
      "description": "Output type for solar position calculation",
      "oneOf": [
        {
          "description": "Output as unit normal vector (x, y, z) in ENU coordinate system",
          "type": "string",
          "enum": [
            "unitNormalVector"
          ]
        },
        {
          "description": "Output as altitude and azimuth angles in degrees",
          "type": "string",
          "enum": [
            "altitudeAndAzimuth"
          ]
        },
        {
          "description": "Output both unit normal vector and altitude/azimuth angles",
          "type": "string",
          "enum": [
            "both"
          ]
        }
      ]
    },
    "StepUnit": {
      "type": "string",
      "enum": [
        "second",
        "minute",
        "hour",
        "day"
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* rejected
### Category
* PLATEAU

## PLATEAU4.SolidIntersectionTestPairCreator
### Type
* processor
### Description
Creates pairs of features from Area On Area Overlayer output for solid intersection testing
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "SolidIntersectionTestPairCreatorParam",
  "type": "object",
  "properties": {
    "pairIdAttribute": {
      "description": "Attribute name to store the pair ID (default: \"pair_id\")",
      "default": "pair_id",
      "type": "string"
    },
    "listAttribute": {
      "description": "Attribute name containing the list of overlapping features from Area On Area Overlayer (default: \"list\")",
      "default": "list",
      "type": "string"
    },
    "gmlIdAttribute": {
      "description": "Attribute name for the GML ID within the list items (default: \"gmlId\")",
      "default": "gmlId",
      "type": "string"
    }
  }
}
```
### Input Ports
* features
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
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
* features
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
  "description": "Configuration for extracting UDX folder structure information from PLATEAU CityGML paths.",
  "type": "object",
  "required": [
    "cityGmlPath"
  ],
  "properties": {
    "cityGmlPath": {
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
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
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
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
* features
### Output Ports
* summary
* unMatchedXlinkFrom
* unMatchedXlinkTo
### Category
* PLATEAU

## PLATEAU4.UnsharedEdgeDetector
### Type
* processor
### Description
Detect unshared edges in triangular meshes - edges that appear only once. REQUIRES: Input geometries must be in a projected coordinate system (meters). Use HorizontalReprojector before this action if input is in geographic coordinates.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "UnsharedEdgeDetector Parameters",
  "description": "Configure unshared edge detection behavior",
  "type": "object",
  "properties": {
    "tolerance": {
      "description": "Tolerance for edge matching in meters (default: 0.1) Edges within this distance are considered the same edge",
      "default": 0.1,
      "type": "number",
      "format": "double"
    },
    "groupBy": {
      "description": "Group By Attributes. When specified, edge detection is performed independently within each group. Features with the same values for these attributes are grouped together.",
      "default": null,
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
* features
### Output Ports
* unshared
### Category
* PLATEAU

## PLATEAU6.BuildingPartConnectivityChecker
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
    "partIdAttribute": {
      "title": "Part ID Attribute",
      "description": "Attribute containing the BuildingPart ID (default: \"featureId\")",
      "default": "featureId",
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
    "fileIndexAttribute": {
      "title": "File Index Attribute",
      "description": "Attribute containing the file index (default: \"fileIndex\")",
      "default": "fileIndex",
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
* features
### Output Ports
* features
### Category
* Feature
* PLATEAU

## PLATEAU6.BuildingUsageAttributeValidator
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
  "required": [
    "codelistsPath"
  ],
  "properties": {
    "codelistsPath": {
      "description": "Expression evaluating to the PLATEAU codelists directory path.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
* l0405BldgError
* cityCodeError
* features
### Category
* PLATEAU

## PLATEAU6.DestinationMeshCodeExtractor
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
    },
    "epsgCode": {
      "title": "EPSG Code",
      "description": "Japanese Plane Rectangular Coordinate System EPSG code for area calculation",
      "default": {
        "type": "flowExpr",
        "value": "6691"
      },
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* rejected
### Category
* PLATEAU

## PLATEAU6.DomainOfDefinitionValidator
### Type
* processor
### Description
Validates domain of definition of CityGML features
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DomainOfDefinitionValidator Parameters",
  "description": "Configuration for validating domain of definition of CityGML features.",
  "type": "object",
  "properties": {
    "codelistsPath": {
      "description": "Fallback codelists directory path expression. When codelists files are not found at the location relative to the GML file, this path will be used as the base directory for resolving codeSpace references.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* rejected
* duplicateGmlIdStats
### Category
* PLATEAU

## PLATEAU6.MissingAttributeDetector
### Type
* processor
### Description
Detect missing attributes in PLATEAU features
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "MissingAttributeDetector Parameters",
  "description": "Configuration for detecting missing attributes in PLATEAU features.",
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
* features
### Output Ports
* summary
* required
* target
* dataQualityC07
* dataQualityC08
### Category
* PLATEAU

## PLATEAU6.ObjectListExtractor
### Type
* processor
### Description
Extract object list
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ObjectListExtractor Parameters",
  "description": "Configuration for extracting object lists from PLATEAU data.",
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
* features
### Output Ports
* features
### Category
* PLATEAU

## PLATEAU6.SolidIntersectionTestPairCreator
### Type
* processor
### Description
Creates pairs of features from Area On Area Overlayer output for solid intersection testing
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "SolidIntersectionTestPairCreatorParam",
  "type": "object",
  "properties": {
    "pairIdAttribute": {
      "description": "Attribute name to store the pair ID (default: \"pair_id\")",
      "default": "pair_id",
      "type": "string"
    },
    "listAttribute": {
      "description": "Attribute name containing the list of overlapping features from Area On Area Overlayer (default: \"list\")",
      "default": "list",
      "type": "string"
    },
    "gmlIdAttribute": {
      "description": "Attribute name for the GML ID within the list items (default: \"gmlId\")",
      "default": "gmlId",
      "type": "string"
    }
  }
}
```
### Input Ports
* features
### Output Ports
* A
* B
### Category
* PLATEAU

## PLATEAU6.UDXFolderExtractor
### Type
* processor
### Description
Extracts UDX folders from cityGML path
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "UDXFolderExtractor Parameters",
  "description": "Configuration for extracting UDX folder structure information from PLATEAU CityGML paths.",
  "type": "object",
  "required": [
    "cityGmlPath"
  ],
  "properties": {
    "cityGmlPath": {
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
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
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* rejected
### Category
* PLATEAU

## PLATEAU6.UnmatchedXlinkDetector
### Type
* processor
### Description
Detect unmatched Xlinks for PLATEAU
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* summary
* unMatchedXlinkFrom
* unMatchedXlinkTo
### Category
* PLATEAU

## Planarity Filter
### Type
* processor
### Description
Filter Features by Geometry Planarity
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Planarity Filter Parameters",
  "description": "Configure how to filter features based on geometry planarity",
  "type": "object",
  "required": [
    "threshold"
  ],
  "properties": {
    "filterType": {
      "title": "Filter Type",
      "description": "The method to use for planarity detection",
      "default": "covariance",
      "allOf": [
        {
          "$ref": "#/definitions/PlanarityFilterType"
        }
      ]
    },
    "threshold": {
      "title": "Threshold",
      "description": "The threshold value for planarity check. For covariance mode: the maximum allowed smallest eigenvalue of the covariance matrix. For height mode: the maximum allowed convex hull minimum height.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  },
  "definitions": {
    "PlanarityFilterType": {
      "description": "Filter type for planarity check",
      "oneOf": [
        {
          "description": "Uses covariance matrix eigenvalue analysis",
          "type": "string",
          "enum": [
            "covariance"
          ]
        },
        {
          "description": "Uses minimum height of the 3D convex hull",
          "type": "string",
          "enum": [
            "height"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* planarity
* notplanarity
### Category
* Geometry

## Polygon Normal Extractor
### Type
* processor
### Description
Extracts the surface normal, signed 2D area, slope and azimuth from polygon features and stores them as attributes.
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* features
* rejected
### Category
* Geometry

## Python Script Processor
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
    "script": {
      "title": "Inline Script",
      "description": "Python script code to execute inline",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "pythonFile": {
      "title": "Python File",
      "description": "Path to a Python script file (supports file://, http://, https://, gs://, etc.)",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "pythonPath": {
      "title": "Python Path",
      "description": "Path to Python interpreter executable (default: python3)",
      "type": [
        "string",
        "null"
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
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Script
* Python

## Ray Intersector
### Type
* processor
### Description
Computes intersection points between rays and geometries.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Ray Intersector Parameters",
  "description": "Configure how rays and geometries are paired and how intersection results are output.",
  "type": "object",
  "required": [
    "ray"
  ],
  "properties": {
    "ray": {
      "title": "Ray",
      "description": "Attributes on the ray features that hold the ray's origin and direction.",
      "allOf": [
        {
          "$ref": "#/definitions/RayDefinition"
        }
      ]
    },
    "pairId": {
      "title": "Pair ID",
      "description": "Expression producing the key that pairs rays with geometries, so that only rays and geometries whose keys match are tested against each other. Omit it to test every ray against every geometry.",
      "default": null,
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "closestIntersectionOnly": {
      "title": "Closest Intersection Only",
      "description": "Whether to keep only the nearest hit per ray rather than every hit along it. Defaults to true.",
      "default": true,
      "type": "boolean"
    },
    "includeRayOrigin": {
      "title": "Include Ray Origin",
      "description": "Whether an intersection at the ray's own origin counts as a hit. When false, intersections nearer than the tolerance are discarded. Defaults to true.",
      "default": true,
      "type": "boolean"
    },
    "outputGeometryType": {
      "title": "Output Geometry Type",
      "description": "Geometry to emit for each intersection.",
      "default": "pointOfIntersection",
      "allOf": [
        {
          "$ref": "#/definitions/OutputGeometryType"
        }
      ]
    },
    "geomId": {
      "title": "Geometry ID",
      "description": "Expression evaluated on each geometry feature to label it. When set, every intersection carries a `geom_id` attribute naming the geometry it hit.",
      "default": null,
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "tolerance": {
      "title": "Tolerance",
      "description": "Distance below which an intersection is treated as coincident with the ray's origin. Defaults to 1e-10.",
      "default": 1e-10,
      "type": "number",
      "format": "double"
    }
  },
  "definitions": {
    "RayDefinition": {
      "title": "Ray Definition",
      "description": "Attributes that hold a ray's origin and direction.",
      "type": "object",
      "required": [
        "dirX",
        "dirY",
        "dirZ",
        "posX",
        "posY",
        "posZ"
      ],
      "properties": {
        "posX": {
          "title": "Origin X Attribute",
          "description": "Attribute holding the X coordinate of the ray's origin.",
          "allOf": [
            {
              "$ref": "#/definitions/Attribute"
            }
          ]
        },
        "posY": {
          "title": "Origin Y Attribute",
          "description": "Attribute holding the Y coordinate of the ray's origin.",
          "allOf": [
            {
              "$ref": "#/definitions/Attribute"
            }
          ]
        },
        "posZ": {
          "title": "Origin Z Attribute",
          "description": "Attribute holding the Z coordinate of the ray's origin.",
          "allOf": [
            {
              "$ref": "#/definitions/Attribute"
            }
          ]
        },
        "dirX": {
          "title": "Direction X Attribute",
          "description": "Attribute holding the X component of the ray's direction.",
          "allOf": [
            {
              "$ref": "#/definitions/Attribute"
            }
          ]
        },
        "dirY": {
          "title": "Direction Y Attribute",
          "description": "Attribute holding the Y component of the ray's direction.",
          "allOf": [
            {
              "$ref": "#/definitions/Attribute"
            }
          ]
        },
        "dirZ": {
          "title": "Direction Z Attribute",
          "description": "Attribute holding the Z component of the ray's direction.",
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
    "OutputGeometryType": {
      "description": "Output geometry type for ray intersection results",
      "oneOf": [
        {
          "title": "Point of Intersection",
          "description": "Emits a point at the location where the ray meets the geometry.",
          "type": "string",
          "enum": [
            "pointOfIntersection"
          ]
        },
        {
          "title": "Line Segment to Intersection",
          "description": "Emits a line running from the ray's origin to the intersection point.",
          "type": "string",
          "enum": [
            "lineSegmentToIntersection"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* ray
* geom
### Output Ports
* intersection
* no-intersection
* rejected
### Category
* Geometry

## Refiner
### Type
* processor
### Description
Refines complex geometry types into simpler primitives.
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* features
### Category
* Geometry

## Rotator 3D
### Type
* processor
### Description
Rotate a 3D polygon using from/to vectors or axis-angle specification
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Rotator 3D Parameters",
  "description": "Configure the rotation for a 3D polygon",
  "type": "object",
  "required": [
    "rotation"
  ],
  "properties": {
    "rotation": {
      "title": "Rotation",
      "description": "The rotation specification: either from/to vectors or axis-angle",
      "allOf": [
        {
          "$ref": "#/definitions/RotationParam"
        }
      ]
    }
  },
  "definitions": {
    "RotationParam": {
      "description": "Rotation specification",
      "oneOf": [
        {
          "description": "Rotation defined by two vectors (from and to)",
          "type": "object",
          "required": [
            "fromX",
            "fromY",
            "fromZ",
            "toX",
            "toY",
            "toZ",
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "fromToVectors"
              ]
            },
            "fromX": {
              "description": "X component of the source direction vector",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            },
            "fromY": {
              "description": "Y component of the source direction vector",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            },
            "fromZ": {
              "description": "Z component of the source direction vector",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            },
            "toX": {
              "description": "X component of the target direction vector",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            },
            "toY": {
              "description": "Y component of the target direction vector",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            },
            "toZ": {
              "description": "Z component of the target direction vector",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            }
          }
        },
        {
          "description": "Rotation defined by an axis and angle",
          "type": "object",
          "required": [
            "angle",
            "axisX",
            "axisY",
            "axisZ",
            "type"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "axisAngle"
              ]
            },
            "axisX": {
              "description": "X component of the rotation axis",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            },
            "axisY": {
              "description": "Y component of the rotation axis",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            },
            "axisZ": {
              "description": "Z component of the rotation axis",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            },
            "angle": {
              "description": "Rotation angle in degrees",
              "type": "object",
              "format": "code",
              "required": [
                "type",
                "value"
              ],
              "properties": {
                "type": {
                  "type": "string",
                  "enum": [
                    "flowExpr"
                  ]
                },
                "value": {
                  "type": "string"
                }
              }
            }
          }
        }
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* rejected
### Category
* Geometry

## SQL Reader
### Type
* source
### Description
Reads features from a SQL database.
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
    "sql": {
      "title": "SQL Query",
      "description": "SQL query expression to execute for retrieving data",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "databaseUrl": {
      "title": "Database URL",
      "description": "Database connection URL (e.g. `sqlite:///tests/sqlite/sqlite.db`, `mysql://user:password@localhost:3306/db`, `postgresql://user:password@localhost:5432/db`)",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
### Output Ports
* features
### Category
* Input

## Shapefile Reader
### Type
* source
### Description
Reads features from a shapefile packaged in a ZIP archive. The archive is expected to hold the .shp, .shx and .dbf files the format defines, though a missing .shx is tolerated.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Shapefile Reader Parameters",
  "description": "Sets which archive is read, the encoding of its attribute table, and whether elevations are kept. Components are paired by name, and the first shapefile is read when the archive holds more than one.",
  "type": "object",
  "properties": {
    "encoding": {
      "title": "Character Encoding",
      "description": "Character encoding for attribute data in the DBF file, such as \"UTF-8\", \"Shift-JIS\", or \"Windows-1252\"; labels are case-insensitive. When omitted, the encoding is taken from the .cpg file if present, else from the code page the .dbf header declares, otherwise UTF-8 (UTF-16 is not supported).",
      "type": [
        "string",
        "null"
      ]
    },
    "force2D": {
      "title": "Force 2D",
      "description": "If true, drops elevations and reads every geometry as 2D. The read fails on a multipatch, which describes a surface in space and has no 2D form.",
      "default": false,
      "type": "boolean"
    },
    "allowEmptyPath": {
      "title": "Allow Empty Path",
      "description": "If true, a dataset path that is empty or null yields no features instead of failing, allowing an optional shapefile input.",
      "default": false,
      "type": "boolean"
    },
    "dataset": {
      "title": "File Path",
      "description": "Expression that returns the path to the input file, either a literal path or a variable reference.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
### Output Ports
* features
### Category
* Input

## Shapefile Writer
### Type
* sink
### Description
Writes features as shapefiles, optionally grouping them into a separate file set per group.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Shapefile Writer Parameters",
  "description": "Sets where the shapefiles are written, whether each is archived, and how features are grouped into separate file sets.",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "output": {
      "title": "Output Directory",
      "description": "Directory the shapefiles are written to, as a path or an expression. Each file set inside it is named after its group value, or \"null\" when no grouping is configured.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "compressOutput": {
      "title": "Compressed Output Directory",
      "description": "Optional directory where each Shapefile is written as its own ZIP archive, holding that Shapefile's .shp, .shx, .dbf, .cpg and .prj, instead of as loose files. Leave unset to write loose files.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "groupBy": {
      "title": "Group By",
      "description": "Attributes to group features by, writing one file set per distinct group and naming it after the group's value. When unset, every feature goes to a single file set.",
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
* features
### Output Ports
### Category
* Output

## Solid Boundary Validator
### Type
* processor
### Description
Validates the Solid Boundary Geometry
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Solid Boundary Validator Parameters",
  "description": "Configure validation parameters for solid boundary geometry",
  "type": "object",
  "required": [
    "tolerance"
  ],
  "properties": {
    "tolerance": {
      "title": "Tolerance",
      "description": "Tolerance value for geometry operations (as an expression evaluating to f64). Used for vertex merging and face triangulation.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
* success
* failed
* rejected
### Category
* Geometry

## Spatial Filter
### Type
* processor
### Description
Filters candidate features by their spatial relationship to filter geometries, tested in the horizontal plane — a 3D geometry is compared by its footprint and must be in a coordinate frame with linear units.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Spatial Filter Parameters",
  "description": "Configure spatial relationship testing between filter and candidate geometries",
  "type": "object",
  "properties": {
    "predicate": {
      "title": "Spatial Predicate",
      "description": "The spatial relationship to test, with the candidate as the subject: `within` passes candidates lying inside a filter geometry, `contains` passes candidates that contain one.",
      "default": "intersects",
      "allOf": [
        {
          "$ref": "#/definitions/SpatialPredicate"
        }
      ]
    },
    "matchMode": {
      "title": "Match Mode",
      "description": "Whether a candidate passes by matching any single filter feature, or only by matching every filter feature.",
      "default": "any",
      "allOf": [
        {
          "$ref": "#/definitions/MatchMode"
        }
      ]
    },
    "mergeFilterAttributes": {
      "title": "Merge Filter Attributes",
      "description": "If true, copies attributes from every matched filter feature onto passing candidates. When multiple matched filters share an attribute, the last matching filter's value wins.",
      "default": false,
      "type": "boolean"
    },
    "mergedAttributesPrefix": {
      "title": "Merged Attributes Prefix",
      "description": "Optional prefix applied to merged filter attribute names to avoid collisions. For example, a prefix of \"filter_\" turns a filter attribute \"zone\" into \"filter_zone\".",
      "default": null,
      "type": [
        "string",
        "null"
      ]
    },
    "outputMatchCountAttribute": {
      "title": "Output Match Count Attribute",
      "description": "Optional attribute name to store the number of filter features the candidate matched.",
      "default": null,
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
    "SpatialPredicate": {
      "oneOf": [
        {
          "description": "Candidate completely contains the filter geometry",
          "type": "string",
          "enum": [
            "contains"
          ]
        },
        {
          "description": "Candidate completely within filter geometry",
          "type": "string",
          "enum": [
            "within"
          ]
        },
        {
          "description": "Geometries have any intersection",
          "type": "string",
          "enum": [
            "intersects"
          ]
        },
        {
          "description": "Geometries have no spatial relationship",
          "type": "string",
          "enum": [
            "disjoint"
          ]
        },
        {
          "description": "Geometries touch at boundaries but don't overlap",
          "type": "string",
          "enum": [
            "touches"
          ]
        },
        {
          "description": "Geometries cross each other",
          "type": "string",
          "enum": [
            "crosses"
          ]
        },
        {
          "description": "Geometries overlap partially",
          "type": "string",
          "enum": [
            "overlaps"
          ]
        },
        {
          "description": "Candidate is covered by filter geometry",
          "type": "string",
          "enum": [
            "coveredBy"
          ]
        },
        {
          "description": "Candidate covers the filter geometry",
          "type": "string",
          "enum": [
            "covers"
          ]
        }
      ]
    },
    "MatchMode": {
      "title": "Match Mode",
      "description": "How the tests against the individual filter features combine into pass or fail.",
      "oneOf": [
        {
          "title": "Any",
          "description": "Passes a candidate that matches at least one filter feature.",
          "type": "string",
          "enum": [
            "any"
          ]
        },
        {
          "title": "All",
          "description": "Passes a candidate only when every filter feature matches it.",
          "type": "string",
          "enum": [
            "all"
          ]
        }
      ]
    },
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* filter
* candidate
### Output Ports
* passed
* failed
* rejected
### Category
* Filter

## Statistics Calculator
### Type
* processor
### Description
Groups features by one or more attributes and computes an aggregate statistic (count, sum, minimum, maximum, or mean) for each group, emitting one feature per group.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Statistics Calculator Parameters",
  "description": "Defines the grouping attributes and the statistics computed for each group.",
  "type": "object",
  "required": [
    "calculations"
  ],
  "properties": {
    "calculations": {
      "title": "Calculations",
      "description": "Statistics to compute for each group. Each entry names an output attribute, the aggregation method, and (except for count) the expression whose values are aggregated.",
      "type": "array",
      "items": {
        "$ref": "#/definitions/Calculation"
      }
    },
    "groupBy": {
      "title": "Group By",
      "description": "Attributes to group features by before aggregating. When omitted, all input features form a single group.",
      "type": [
        "array",
        "null"
      ],
      "items": {
        "$ref": "#/definitions/Attribute"
      }
    },
    "groupId": {
      "title": "Group ID",
      "description": "Optional attribute in which to store the group identifier, formed by joining the Group By values with '|'.",
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
    "Calculation": {
      "type": "object",
      "required": [
        "newAttribute"
      ],
      "properties": {
        "newAttribute": {
          "title": "New Attribute Name",
          "description": "Name of the output attribute that stores the computed statistic.",
          "allOf": [
            {
              "$ref": "#/definitions/Attribute"
            }
          ]
        },
        "aggregation": {
          "title": "Aggregation Method",
          "description": "Statistic to compute across each group. Defaults to sum.",
          "default": "sum",
          "allOf": [
            {
              "$ref": "#/definitions/AggregationMethod"
            }
          ]
        },
        "expr": {
          "title": "Value Expression",
          "description": "Expression evaluated per feature to produce the value being aggregated. Required for every method except count, which counts features regardless of value.",
          "type": [
            "object",
            "null"
          ],
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        }
      }
    },
    "Attribute": {
      "type": "string"
    },
    "AggregationMethod": {
      "oneOf": [
        {
          "title": "Count",
          "description": "Counts the number of features in each group. The value expression is not required and is ignored.",
          "type": "string",
          "enum": [
            "count"
          ]
        },
        {
          "title": "Sum",
          "description": "Adds the value expression across all features in each group.",
          "type": "string",
          "enum": [
            "sum"
          ]
        },
        {
          "title": "Minimum",
          "description": "Keeps the smallest value of the expression in each group.",
          "type": "string",
          "enum": [
            "min"
          ]
        },
        {
          "title": "Maximum",
          "description": "Keeps the largest value of the expression in each group.",
          "type": "string",
          "enum": [
            "max"
          ]
        },
        {
          "title": "Mean",
          "description": "Averages the value of the expression across all features in each group.",
          "type": "string",
          "enum": [
            "mean"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
* complete
### Category
* Attribute

## Three Dimension Box Replacer
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
    },
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
* features
### Output Ports
* features
### Category
* Geometry

## Three Dimension Forcer
### Type
* processor
### Description
Adds Z-coordinates to 2D geometries to produce 3D output.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Three Dimension Forcer Parameters",
  "description": "Configure the elevation applied to 2D geometry and how existing Z values are treated.",
  "type": "object",
  "properties": {
    "elevation": {
      "title": "Elevation",
      "description": "Z-coordinate applied to every point, as a constant or an expression. Defaults to 0.0.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "preserveExistingZ": {
      "title": "Preserve Existing Z Values",
      "description": "Whether geometry that is already 3D passes through untouched. Defaults to true, so existing Z is kept. Set it to false to overwrite every Z value with the elevation.",
      "default": true,
      "type": "boolean"
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Geometry

## Three Dimension Planarity Rotator
### Type
* processor
### Description
Rotates a single or a set of 2D geometries in 3D space to align them horizontally.
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* features
* rejected
### Category
* Geometry

## Three Dimension Rotator
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
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "originX": {
      "title": "Origin X",
      "description": "X coordinate of the rotation origin point",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "originY": {
      "title": "Origin Y",
      "description": "Y coordinate of the rotation origin point",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "originZ": {
      "title": "Origin Z",
      "description": "Z coordinate of the rotation origin point",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "directionX": {
      "title": "Direction X",
      "description": "X component of the rotation axis direction vector",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "directionY": {
      "title": "Direction Y",
      "description": "Y component of the rotation axis direction vector",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "directionZ": {
      "title": "Direction Z",
      "description": "Z component of the rotation axis direction vector",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Geometry

## Two Dimension Forcer
### Type
* processor
### Description
Removes Z-coordinates from 3D geometries to produce 2D output.
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* features
* rejected
### Category
* Geometry

## Vertex Counter
### Type
* processor
### Description
Count Geometry Vertices to Attribute
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Vertex Counter Parameters",
  "description": "Configure where to store the count of vertices found in geometries",
  "type": "object",
  "required": [
    "outputAttribute"
  ],
  "properties": {
    "outputAttribute": {
      "title": "Output Attribute",
      "description": "Name of the attribute where the vertex count will be stored as a number",
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
* features
### Output Ports
* features
### Category
* Geometry

## Vertex Remover
### Type
* processor
### Description
Remove Redundant Vertices from Geometry
### Parameters
* No parameters
### Input Ports
* features
### Output Ports
* features
* rejected
### Category
* Geometry

## Vertical Reprojector
### Type
* processor
### Description
Reprojects the vertical coordinate of feature geometry between vertical datums.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Vertical Reprojector Parameters",
  "description": "Configure which vertical datum conversion to apply.",
  "type": "object",
  "required": [
    "reprojectorType"
  ],
  "properties": {
    "reprojectorType": {
      "title": "Reprojector Type",
      "description": "Vertical datum conversion applied to each geometry's Z coordinate.",
      "allOf": [
        {
          "$ref": "#/definitions/VerticalReprojectorType"
        }
      ]
    }
  },
  "definitions": {
    "VerticalReprojectorType": {
      "oneOf": [
        {
          "title": "JGD2011 to WGS 84",
          "description": "Converts JGD2011 orthometric heights to WGS 84 ellipsoidal heights using the Japanese geoid model.",
          "type": "string",
          "enum": [
            "jgd2011ToWgs84"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Geometry

## XML Fragmenter
### Type
* processor
### Description
Splits an XML document into features by matching element names, emitting one feature per matched element with its XML text, tag name, and element and parent identifiers as attributes.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "XML Fragmenter Parameters",
  "description": "Configures where the XML document comes from and which of its elements become features.",
  "oneOf": [
    {
      "title": "XML File",
      "description": "Reads the XML document from the file path or URL held in the attribute.",
      "type": "object",
      "required": [
        "attribute",
        "elementsToExclude",
        "elementsToMatch",
        "source"
      ],
      "properties": {
        "source": {
          "type": "string",
          "enum": [
            "file"
          ]
        },
        "elementsToMatch": {
          "title": "Elements to Match",
          "description": "Expression returning the list of element names whose subtrees are emitted as fragments. Names include the namespace prefix as written in the document, such as `bldg:Building`.",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "elementsToExclude": {
          "title": "Elements to Exclude",
          "description": "Expression returning the list of element names to skip even when they also appear in the matched elements. Return an empty list to match everything.",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "attribute": {
          "title": "XML Attribute",
          "description": "Attribute holding the XML document: the file path or URL to read when the source is a file, or the XML text itself when the source is text.",
          "allOf": [
            {
              "$ref": "#/definitions/Attribute"
            }
          ]
        }
      }
    },
    {
      "title": "XML Text",
      "description": "Uses the attribute value itself as the XML document.",
      "type": "object",
      "required": [
        "attribute",
        "elementsToExclude",
        "elementsToMatch",
        "source"
      ],
      "properties": {
        "source": {
          "type": "string",
          "enum": [
            "text"
          ]
        },
        "elementsToMatch": {
          "title": "Elements to Match",
          "description": "Expression returning the list of element names whose subtrees are emitted as fragments. Names include the namespace prefix as written in the document, such as `bldg:Building`.",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "elementsToExclude": {
          "title": "Elements to Exclude",
          "description": "Expression returning the list of element names to skip even when they also appear in the matched elements. Return an empty list to match everything.",
          "type": "object",
          "format": "code",
          "required": [
            "type",
            "value"
          ],
          "properties": {
            "type": {
              "type": "string",
              "enum": [
                "flowExpr"
              ]
            },
            "value": {
              "type": "string"
            }
          }
        },
        "attribute": {
          "title": "XML Attribute",
          "description": "Attribute holding the XML document: the file path or URL to read when the source is a file, or the XML text itself when the source is text.",
          "allOf": [
            {
              "$ref": "#/definitions/Attribute"
            }
          ]
        }
      }
    }
  ],
  "definitions": {
    "Attribute": {
      "type": "string"
    }
  }
}
```
### Input Ports
* features
### Output Ports
* features
### Category
* Transform

## XML Validator
### Type
* processor
### Description
Validates XML documents for well-formed syntax, namespace declarations, or compliance with the XSD schemas they reference. Details of any errors found are added to an `xmlError` attribute.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "XML Validator Parameters",
  "description": "Configures which XML document is validated and how thoroughly.",
  "type": "object",
  "required": [
    "attribute",
    "inputType",
    "validationType"
  ],
  "properties": {
    "attribute": {
      "title": "XML Attribute",
      "description": "Attribute holding the XML document: the file path or URL to read, or the XML text itself, depending on the input type.",
      "allOf": [
        {
          "$ref": "#/definitions/Attribute"
        }
      ]
    },
    "inputType": {
      "title": "Input Type",
      "description": "Whether the attribute holds the location of the document or the document itself.",
      "allOf": [
        {
          "$ref": "#/definitions/XmlInputType"
        }
      ]
    },
    "validationType": {
      "title": "Validation Type",
      "description": "Checks to run against the document.",
      "allOf": [
        {
          "$ref": "#/definitions/ValidationType"
        }
      ]
    }
  },
  "definitions": {
    "Attribute": {
      "type": "string"
    },
    "XmlInputType": {
      "oneOf": [
        {
          "title": "XML File",
          "description": "Reads the document from the file path or URL held in the attribute.",
          "type": "string",
          "enum": [
            "file"
          ]
        },
        {
          "title": "XML Text",
          "description": "Uses the attribute value itself as the document.",
          "type": "string",
          "enum": [
            "text"
          ]
        }
      ]
    },
    "ValidationType": {
      "oneOf": [
        {
          "title": "Syntax",
          "description": "Checks that the document is well-formed XML.",
          "type": "string",
          "enum": [
            "syntax"
          ]
        },
        {
          "title": "Syntax and Namespace",
          "description": "Also checks that every namespace prefix used by an element is declared on that element or an ancestor, and that unprefixed elements have a default namespace.",
          "type": "string",
          "enum": [
            "syntaxAndNamespace"
          ]
        },
        {
          "title": "Syntax and Schema",
          "description": "Also validates the document against the XSD schemas named by its `xsi:schemaLocation`. Unreachable remote schemas are skipped, as those locations are hints per the XML Schema specification.",
          "type": "string",
          "enum": [
            "syntaxAndSchema"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
* success
* failed
### Category
* Transform

## XML Writer
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
      "title": "Output File",
      "description": "Output path or expression for the XML file to create.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
### Category
* Output

## Zip File Writer
### Type
* sink
### Description
Compresses files referenced by incoming features into a single ZIP archive.
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
      "title": "Output File",
      "description": "Output path or expression for the ZIP archive to create.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  }
}
```
### Input Ports
* features
### Output Ports
### Category
* Output

## glTF Reader
### Type
* source
### Description
Reads 3D models from glTF 2.0 files, including meshes, nodes, scenes, and geometry primitives.
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "glTF Reader Parameters",
  "type": "object",
  "properties": {
    "mergeMeshes": {
      "title": "Merge Meshes",
      "description": "Combines all meshes from the glTF file into a single output feature.",
      "default": false,
      "type": "boolean"
    },
    "includeNodes": {
      "title": "Include Nodes",
      "description": "Includes node hierarchy information from the glTF scene graph in feature attributes.",
      "default": true,
      "type": "boolean"
    },
    "featureClassAttribute": {
      "title": "Feature Class Attribute",
      "description": "Attribute key to store the EXT_structural_metadata class name under, for each split feature. If unset, the class name is not added as an attribute.",
      "default": null,
      "type": [
        "string",
        "null"
      ]
    },
    "featureGranularity": {
      "title": "Feature Granularity",
      "description": "What one output feature represents. `mesh` (the default) emits one feature per glTF mesh. `featureId` splits each mesh into one feature per EXT_mesh_features feature ID, attaching that object's EXT_structural_metadata properties; files without EXT_mesh_features still yield one feature per mesh.",
      "default": "mesh",
      "allOf": [
        {
          "$ref": "#/definitions/FeatureGranularity"
        }
      ]
    },
    "dataset": {
      "title": "File Path",
      "description": "Expression that returns the path to the input file, either a literal path or a variable reference.",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "inline": {
      "title": "Inline Content",
      "description": "Expression that returns the file content as text instead of reading from a file path",
      "type": [
        "object",
        "null"
      ],
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    }
  },
  "definitions": {
    "FeatureGranularity": {
      "title": "Feature Granularity",
      "description": "Controls what one output feature represents.",
      "oneOf": [
        {
          "description": "One feature per glTF mesh (or per node instance). Structural-metadata properties are not surfaced, because they are per object, not per mesh.",
          "type": "string",
          "enum": [
            "mesh"
          ]
        },
        {
          "description": "One feature per `EXT_mesh_features` feature ID, each carrying that object's `EXT_structural_metadata` properties. Files without `EXT_mesh_features` still yield one feature per mesh.",
          "type": "string",
          "enum": [
            "featureId"
          ]
        }
      ]
    }
  }
}
```
### Input Ports
### Output Ports
* features
### Category
* Input

## glTF Writer
### Type
* sink
### Description
Writes 3D features to GLTF format with optional texture attachment
### Parameters
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "glTF Writer Parameters",
  "description": "Configuration for writing features to GLTF 3D format.",
  "type": "object",
  "required": [
    "output"
  ],
  "properties": {
    "output": {
      "description": "Output file path. When `schemaKey` is set, treated as a directory and each feature type is written to `<output>/<schemaKeyValue>.glb`; otherwise all features are written to this single file.",
      "type": "object",
      "format": "code",
      "required": [
        "type",
        "value"
      ],
      "properties": {
        "type": {
          "type": "string",
          "enum": [
            "flowExpr",
            "string"
          ]
        },
        "value": {
          "type": "string"
        }
      }
    },
    "attachTexture": {
      "description": "Whether to attach texture information to the GLTF model",
      "type": [
        "boolean",
        "null"
      ]
    },
    "dracoCompression": {
      "description": "Apply Draco compression to the geometry",
      "type": [
        "boolean",
        "null"
      ]
    },
    "schemaKey": {
      "description": "Features are grouped by this attribute and written to separate files. The key is excluded from output attributes.",
      "type": [
        "string",
        "null"
      ]
    }
  }
}
```
### Input Ports
* features
### Output Ports
### Category
* File
