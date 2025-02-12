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
				"type":        "string",
				"description": "Authentication token for the execution",
			},
			"notificationUrl": map[string]interface{}{
				"type":        "string",
				"format":      "uri",
				"description": "URL to notify upon completion",
			},
			"with": map[string]interface{}{
				"type":                 "object",
				"description":          "Execution parameters - can contain any valid JSON object properties",
				"additionalProperties": true,
			},
		},
	}

	return json.MarshalIndent(schema, "", "  ")
}
