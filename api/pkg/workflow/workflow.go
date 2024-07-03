package workflow

type Workflow struct {
	id           ID
	name         string
	entryGraphId string
	with         map[string]interface{}
	graphs       []Graph
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

func (w *Workflow) SetID(id ID) {
	w.id = id
}

func (w *Workflow) SetName(name string) {
	w.name = name
}

func (w *Workflow) SetEntryGraphId(entryGraphId string) {
	w.entryGraphId = entryGraphId
}

func (w *Workflow) SetWith(with map[string]interface{}) {
	w.with = with
}

func (w *Workflow) SetGraphs(graphs []Graph) {
	w.graphs = graphs
}
