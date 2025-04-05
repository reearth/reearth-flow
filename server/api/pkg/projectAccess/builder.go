package projectAccess

type Builder struct {
	pa *ProjectAccess
}

func New() *Builder {
	return &Builder{pa: &ProjectAccess{}}
}

func (b *Builder) Build() (*ProjectAccess, error) {
	if b.pa.id.IsNil() {
		return nil, ErrInvalidID
	}
	return b.pa, nil
}

func (b *Builder) MustBuild() *ProjectAccess {
	r, err := b.Build()
	if err != nil {
		panic(err)
	}
	return r
}

func (b *Builder) ID(id ID) *Builder {
	b.pa.id = id
	return b
}

func (b *Builder) NewID() *Builder {
	b.pa.id = NewID()
	return b
}

func (b *Builder) Project(project ProjectID) *Builder {
	b.pa.project = project
	return b
}

func (b *Builder) IsPublic(isPublic bool) *Builder {
	b.pa.isPublic = isPublic
	return b
}

func (b *Builder) Token(token string) *Builder {
	b.pa.token = token
	return b
}

func (b *Builder) UserRoles(userRoles UserRoleList) *Builder {
	b.pa.userRoles = userRoles
	return b
}
