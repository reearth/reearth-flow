package workflow

type Workflow struct {
	id        ID
	project   ProjectID
	workspace WorkspaceID
	// Meta      *string
	url   string
	graph GraphID
}

func NewWorkflow(id ID, project ProjectID, workspace WorkspaceID, url string, graph GraphID) *Workflow {
	return &Workflow{
		id:        id,
		project:   project,
		workspace: workspace,
		url:       url,
		graph:     graph,
	}
}

func (w *Workflow) ID() ID {
	return w.id
}

func (w *Workflow) Project() ProjectID {
	return w.project
}

func (w *Workflow) Workspace() WorkspaceID {
	return w.workspace
}

func (w *Workflow) URL() string {
	return w.url
}

func (w *Workflow) Graph() GraphID {
	return w.graph
}
