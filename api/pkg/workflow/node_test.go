package workflow

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestNode_SetID(t *testing.T) {
	n := &NodeType{}
	n.SetID("id")
	assert.Equal(t, "id", n.id)
}

func TestNode_SetData(t *testing.T) {
	n := &NodeType{}
	n.SetData(Data{
		name:          "name",
		inputs:        []string{"inputs"},
		outputs:       []string{"outputs"},
		transformerID: "transformerID",
		params:        map[string]interface{}{"key": "value"},
	})
	assert.Equal(t, Data{
		name:          "name",
		inputs:        []string{"inputs"},
		outputs:       []string{"outputs"},
		transformerID: "transformerID",
		params:        map[string]interface{}{"key": "value"},
	}, n.data)
}
