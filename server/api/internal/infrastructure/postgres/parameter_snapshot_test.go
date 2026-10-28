package postgres

import (
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestParametersFromJSON_SnapshotWithoutTimestamps(t *testing.T) {
	legacy := []byte(`[{"id":"` + id.NewParameterID().String() + `","project_id":"` + id.NewProjectID().String() +
		`","name":"area","type":"TEXT","index":0,"required":true,"public":false,"default_value":"tokyo"}]`)

	params, err := parametersFromJSON(legacy)

	require.NoError(t, err)
	require.Len(t, params, 1)
	assert.Equal(t, "area", params[0].Name())
	assert.WithinDuration(t, time.Now(), params[0].CreatedAt(), time.Minute)
	assert.WithinDuration(t, time.Now(), params[0].UpdatedAt(), time.Minute)
}
