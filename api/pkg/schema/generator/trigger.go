package generator

import (
	"encoding/json"
)

func GenerateTriggerExecutionSchema() ([]byte, error) {
	schema := map[string]interface{}{
		"$schema": "http://json-schema.org/draft-07/schema#",
		"title":   "TriggerExecution",
		"type":    "object",
		"properties": map[string]interface{}{
			"authToken": map[string]interface{}{
				"type": "string",
			},
			"notificationUrl": map[string]interface{}{
				"type":   "string",
				"format": "uri",
			},
			"with": map[string]interface{}{
				"type": "object",
				"properties": map[string]interface{}{
					"cityGmlPath":   map[string]interface{}{"type": "string"},
					"cityCode":      map[string]interface{}{"type": "string"},
					"codelistsPath": map[string]interface{}{"type": "string"},
					"schemasPath":   map[string]interface{}{"type": "string"},
					"schemaJson":    map[string]interface{}{"type": "string"},
					"targetPackages": map[string]interface{}{
						"type":  "array",
						"items": map[string]interface{}{"type": "string"},
					},
					"addNsprefixToFeatureTypes":      map[string]interface{}{"type": "boolean"},
					"extractDmGeometryAsXmlFragment": map[string]interface{}{"type": "boolean"},
					"outputPath":                     map[string]interface{}{"type": "string"},
				},
			},
		},
		"required": []string{"notificationUrl", "authToken", "with"},
	}

	return json.MarshalIndent(schema, "", "  ")
}
