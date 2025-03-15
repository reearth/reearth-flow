package trigger

import "github.com/reearth/reearth-flow/api/pkg/schema"

const (
	ExecutionSchemaPath = "trigger/execution.json"
)

var ExecutionValidator = schema.NewValidator(ExecutionSchemaPath)
