package workflow

import (
	"log"
	"os"

	"gopkg.in/yaml.v2"
)

type Workflow struct {
	ID         ID `json:"id"`
	Project    ProjectID
	Workspace  WorkspaceID
	YamlString *string
}

func NewWorkflow(id ID, project ProjectID, workspace WorkspaceID, yaml *string) *Workflow {
	return &Workflow{
		ID:         id,
		Project:    project,
		Workspace:  workspace,
		YamlString: yaml,
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
		return nil, err
	}

	fileName := id.String() + "-workflow" + ".yaml"

	f, err := os.CreateTemp("", fileName)
	if err != nil {
		log.Fatal(err)
	}

	defer func() {
		if err := f.Close(); err != nil {
			log.Println("Error closing file:", err)
		}
		if err := os.Remove(f.Name()); err != nil {
			log.Println("Error removing file:", err)
		}
	}()

	if _, err := f.Write(yamlData); err != nil {
		return nil, err
	}

	stringifiedYaml := string(yamlData)
	return &stringifiedYaml, nil
}
