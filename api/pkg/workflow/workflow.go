package workflow

import (
	"fmt"

	"github.com/spf13/afero"
	"gopkg.in/yaml.v2"
)

type Workflow struct {
	ID        ID `json:"id"`
	Project   ProjectID
	Workspace WorkspaceID
	// Meta *string
	URL string
}

func NewWorkflow(id ID, project ProjectID, workspace WorkspaceID, url string) *Workflow {
	return &Workflow{
		ID:        id,
		Project:   project,
		Workspace: workspace,
		URL:       url,
	}
}

func ToWorkflowYaml(id ID, name, entryGraphID string, with *map[string]interface{}, graphs []*Graph) (*string, error) {
	w := map[string]interface{}{
		"id":           id.String(),
		"name":         name,
		"entryGraphID": entryGraphID,
		"with":         with,
		"graphs":       graphs,
	}

	yamlData, err := yaml.Marshal(w)
	if err != nil {
		return nil, fmt.Errorf("error marshaling YAML: %w", err)
	}

	stringifiedYaml := string(yamlData)
	return &stringifiedYaml, nil
}

func WriteWorkflowToFile(fs afero.Fs, id ID, yamlData []byte) error {
	fileName := id.String() + "-workflow.yaml"

	f, err := afero.TempFile(fs, "", fileName)
	if err != nil {
		return fmt.Errorf("error creating temp file: %w", err)
	}

	defer func() {
		closeErr := f.Close()
		if err == nil && closeErr != nil {
			err = fmt.Errorf("error closing file: %w", closeErr)
		}
	}()

	if _, err := f.Write(yamlData); err != nil {
		return fmt.Errorf("error writing to file: %w", err)
	}

	return err
}
