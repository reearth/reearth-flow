package workflow

import "github.com/reearth/reearth-flow/api/pkg/id"

type Edge struct {
	id       id.EdgeID
	from     string
	to       string
	fromPort string
	toPort   string
}

func NewEdge(id id.EdgeID, from, to, fromPort, toPort string) *Edge {
	return &Edge{
		id:       id,
		from:     from,
		to:       to,
		fromPort: fromPort,
		toPort:   toPort,
	}
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
