package schema

import (
	"embed"
	"fmt"

	"github.com/xeipuuv/gojsonschema"
)

//go:embed */*.json
var schemaFS embed.FS

type SchemaValidator interface {
	Validate(data []byte) error
}

type validator struct {
	schemaPath string
}

func NewValidator(schemaPath string) SchemaValidator {
	return &validator{
		schemaPath: schemaPath,
	}
}

func (v *validator) Validate(data []byte) error {
	schemaBytes, err := schemaFS.ReadFile(v.schemaPath)
	if err != nil {
		return fmt.Errorf("failed to load schema: %v", err)
	}

	schemaLoader := gojsonschema.NewStringLoader(string(schemaBytes))
	documentLoader := gojsonschema.NewBytesLoader(data)

	result, err := gojsonschema.Validate(schemaLoader, documentLoader)
	if err != nil {
		return err
	}

	if !result.Valid() {
		var errMsg string
		for _, desc := range result.Errors() {
			errMsg += fmt.Sprintf("- %s\n", desc)
		}
		return fmt.Errorf("validation failed:\n%s", errMsg)
	}

	return nil
}
