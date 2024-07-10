package workflow

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
)

func TestNewEdge(t *testing.T) {
	edgeID := id.NewEdgeID()
	from := "from"
	to := "to"
	fromPort := "fromPort"
	toPort := "toPort"

	result := NewEdge(edgeID, from, to, fromPort, toPort)

	want := &Edge{
		id:       edgeID,
		from:     from,
		to:       to,
		fromPort: fromPort,
		toPort:   toPort,
	}

	assert.Equal(t, result, want)
}
