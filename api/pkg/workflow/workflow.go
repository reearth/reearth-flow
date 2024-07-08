package workflow

type Workflow struct {
	id           ID
	name         string
	entryGraphId string
	with         map[string]interface{}
	graphs       []Graph
}

func NewWorkflow(id ID, name string, entryGraphId string, with map[string]interface{}, graphs []Graph) *Workflow {
	return &Workflow{
		id:           id,
		name:         name,
		entryGraphId: entryGraphId,
		with:         with,
		graphs:       graphs,
	}
}

func (w *Workflow) ID() ID {
	return w.id
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

func (w *Workflow) Graphs() []Graph {
	return w.graphs
}
