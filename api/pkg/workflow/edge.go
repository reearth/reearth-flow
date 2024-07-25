package workflow

import "github.com/reearth/reearth-flow/api/pkg/id"

type Edge struct {
	ID       id.EdgeID
	From     string
	To       string
	FromPort string
	ToPort   string
}

func NewEdge(id id.EdgeID, from, to, fromPort, toPort string) *Edge {
	return &Edge{
		ID:       id,
		From:     from,
		To:       to,
		FromPort: fromPort,
		ToPort:   toPort,
	}
}
