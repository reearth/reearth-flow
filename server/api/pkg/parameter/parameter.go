package parameter

import (
	"time"
)

type Type string

const (
	TypeChoice             Type = "CHOICE"
	TypeColor              Type = "COLOR"
	TypeDatetime           Type = "DATETIME"
	TypeFileFolder         Type = "FILE_FOLDER"
	TypeMessage            Type = "MESSAGE"
	TypeNumber             Type = "NUMBER"
	TypePassword           Type = "PASSWORD"
	TypeText               Type = "TEXT"
	TypeYesNo              Type = "YES_NO"
	TypeAttributeName      Type = "ATTRIBUTE_NAME"
	TypeCoordinateSystem   Type = "COORDINATE_SYSTEM"
	TypeDatabaseConnection Type = "DATABASE_CONNECTION"
	TypeGeometry           Type = "GEOMETRY"
	TypeReprojectionFile   Type = "REPROJECTION_FILE"
	TypeWebConnection      Type = "WEB_CONNECTION"
)

type Parameter struct {
	id           ID
	projectID    ProjectID
	name         string
	typ          Type
	required     bool
	public       bool
	defaultValue interface{}
	index        int
	createdAt    time.Time
	updatedAt    time.Time
}

func (p *Parameter) ID() ID {
	return p.id
}

func (p *Parameter) ProjectID() ProjectID {
	return p.projectID
}

func (p *Parameter) Name() string {
	return p.name
}

func (p *Parameter) Type() Type {
	return p.typ
}

func (p *Parameter) Required() bool {
	return p.required
}

func (p *Parameter) Public() bool {
	return p.public
}

func (p *Parameter) DefaultValue() interface{} {
	return p.defaultValue
}

func (p *Parameter) Index() int {
	return p.index
}

func (p *Parameter) CreatedAt() time.Time {
	return p.createdAt
}

func (p *Parameter) UpdatedAt() time.Time {
	return p.updatedAt
}

func (p *Parameter) SetDefaultValue(v interface{}) {
	p.defaultValue = v
	p.updatedAt = time.Now()
}

func (p *Parameter) SetName(name string) {
	p.name = name
	p.updatedAt = time.Now()
}

func (p *Parameter) SetType(typ Type) {
	p.typ = typ
	p.updatedAt = time.Now()
}

func (p *Parameter) SetRequired(required bool) {
	p.required = required
	p.updatedAt = time.Now()
}

func (p *Parameter) SetPublic(public bool) {
	p.public = public
	p.updatedAt = time.Now()
}

func (p *Parameter) SetIndex(i int) {
	p.index = i
	p.updatedAt = time.Now()
}
