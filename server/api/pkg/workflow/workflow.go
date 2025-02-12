package workflow

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
