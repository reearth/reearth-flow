package workspace

type Workspace struct {
	metadata Metadata
	name     string
	alias    string
	members  []Member
	id       ID
	personal bool
}

type List []*Workspace

func (w *Workspace) ID() ID {
	return w.id
}

func (w *Workspace) Name() string {
	return w.name
}

func (w *Workspace) Alias() string {
	return w.alias
}

func (w *Workspace) Metadata() Metadata {
	return w.metadata
}

func (w *Workspace) Personal() bool {
	return w.personal
}

func (w *Workspace) Members() []Member {
	return w.members
}
