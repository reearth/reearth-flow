package workflow

type Workflow struct {
	id           ID
	workspace    WorkspaceID
	project      ProjectID
	name         string
	entryGraphId string
	with         map[string]interface{}
	graphs       []*Graph
}

func NewWorkflow(id ID, workspace WorkspaceID, project ProjectID, name string, entryGraphId string, with map[string]interface{}, graphs []*Graph) *Workflow {
	return &Workflow{
		id:           id,
		workspace:    workspace,
		project:      project,
		name:         name,
		entryGraphId: entryGraphId,
		with:         with,
		graphs:       graphs,
	}
}

func (w *Workflow) ID() ID {
	return w.id
}

func (w *Workflow) Workspace() WorkspaceID {
	return w.workspace
}

func (w *Workflow) Project() ProjectID {
	return w.project
}

func (w *Workflow) Name() string {
	return w.name
}

func (w *Workflow) EntryGraphId() string {
	return w.entryGraphId
}

func (w *Workflow) With() map[string]interface{} {
	return w.with
}

func (w *Workflow) Graphs() []*Graph {
	return w.graphs
}
