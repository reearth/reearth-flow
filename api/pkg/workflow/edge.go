package workflow

type Edges struct {
	id           string
	source       []string
	sourceHandle string
	target       []string
	targetHandle string
}

func (e *Edges) ID() string {
	return e.id
}

func (e *Edges) Source() []string {
	return e.source
}

func (e *Edges) SourceHandle() string {
	return e.sourceHandle
}

func (e *Edges) Target() []string {
	return e.target
}

func (e *Edges) TargetHandle() string {
	return e.targetHandle
}

func (e *Edges) SetID(id string) {
	e.id = id
}

func (e *Edges) SetSource(source []string) {
	e.source = source
}

func (e *Edges) SetSourceHandle(sourceHandle string) {
	e.sourceHandle = sourceHandle
}

func (e *Edges) SetTarget(target []string) {
	e.target = target
}

func (e *Edges) SetTargetHandle(targetHandle string) {
	e.targetHandle = targetHandle
}
