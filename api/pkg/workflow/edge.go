package workflow

import "github.com/reearth/reearth-flow/api/pkg/id"

type Edge struct {
	id       id.EdgeID
	from     string
	to       string
	fromPort string
	toPort   string
}

func (e *Edge) ID() id.EdgeID {
	return e.id
}

func (e *Edge) From() string {
	return e.from
}

func (e *Edge) To() string {
	return e.to
}

func (e *Edge) FromPort() string {
	return e.fromPort
}

func (e *Edge) ToPort() string {
	return e.toPort
}

func (e *Edge) SetID(id id.EdgeID) {
	e.id = id
}

func (e *Edge) SetFrom(from string) {
	e.from = from
}

func (e *Edge) SetTo(to string) {
	e.to = to
}

func (e *Edge) SetFromPort(fromPort string) {
	e.fromPort = fromPort
}

func (e *Edge) SetToPort(toPort string) {
	e.toPort = toPort
}
