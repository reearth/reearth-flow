package postgres

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/variable"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestVariablesJSONRoundTrip(t *testing.T) {
	in := []variable.Variable{
		{Key: "a", Type: parameter.Type("text"), Value: "hello"},
		{Key: "n", Type: parameter.Type("number"), Value: float64(3)},
	}
	b, err := variablesToJSON(in)
	require.NoError(t, err)
	out, err := variablesFromJSON(b)
	require.NoError(t, err)
	assert.Equal(t, in, out)
}

func TestVariablesJSON_EmptyAndNil(t *testing.T) {
	b, err := variablesToJSON(nil)
	require.NoError(t, err)
	assert.Nil(t, b)
	out, err := variablesFromJSON(nil)
	require.NoError(t, err)
	assert.Nil(t, out)
}
