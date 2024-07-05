package workflow

type Builder struct {
	w *Workflow
}

func New() *Builder {
	return &Builder{w: &Workflow{}}
}

func (b *Builder) Build() (*Workflow, error) {
	if b.w.id.IsNil() {
		return nil, ErrInvalidID
	}
	return b.w, nil
}

func (b *Builder) MustBuild() *Workflow {
	r, err := b.Build()
	if err != nil {
		panic(err)
	}
	return r
}

func (b *Builder) ID(id ID) *Builder {
	b.w.id = id
	return b
}

func (b *Builder) NewID() *Builder {
	b.w.id = NewID()
	return b
}

func (b *Builder) Name(name string) *Builder {
	b.w.name = name
	return b
}

func (b *Builder) EntryGraphID(entryGraphId string) *Builder {
	b.w.entryGraphId = entryGraphId
	return b
}

func (b *Builder) With(with map[string]interface{}) *Builder {
	b.w.with = with
	return b
}

func (b *Builder) Graphs(graphs []Graph) *Builder {
	b.w.graphs = graphs
	return b
}
