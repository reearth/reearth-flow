package postgres

import (
	"encoding/json"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestParametersToJSON(t *testing.T) {
	created := time.Date(2026, 10, 1, 9, 0, 0, 0, time.UTC)
	updated := time.Date(2026, 10, 2, 9, 0, 0, 0, time.UTC)
	pid := id.NewParameterID()
	projID := id.NewProjectID()
	p, err := parameter.New().
		ID(pid).
		ProjectID(projID).
		Name("area").
		Type(parameter.TypeChoice).
		Index(2).
		Required(true).
		Public(true).
		DefaultValue("tokyo").
		Config(map[string]any{"choices": []any{"tokyo", "osaka"}}).
		CreatedAt(created).
		UpdatedAt(updated).
		Build()
	require.NoError(t, err)

	t.Run("writes every field", func(t *testing.T) {
		b, err := parametersToJSON([]*parameter.Parameter{p})
		require.NoError(t, err)

		var docs []map[string]any
		require.NoError(t, json.Unmarshal(b, &docs))
		require.Len(t, docs, 1)
		assert.Equal(t, map[string]any{
			"id":            pid.String(),
			"project_id":    projID.String(),
			"name":          "area",
			"type":          string(parameter.TypeChoice),
			"index":         float64(2),
			"required":      true,
			"public":        true,
			"default_value": "tokyo",
			"config":        map[string]any{"choices": []any{"tokyo", "osaka"}},
			"created_at":    created.Format(time.RFC3339),
			"updated_at":    updated.Format(time.RFC3339),
		}, docs[0])
	})

	t.Run("skips nil parameters", func(t *testing.T) {
		b, err := parametersToJSON([]*parameter.Parameter{nil, p, nil})
		require.NoError(t, err)

		var docs []parameterJSON
		require.NoError(t, json.Unmarshal(b, &docs))
		require.Len(t, docs, 1)
		assert.Equal(t, pid.String(), docs[0].ID)
	})

	t.Run("nil and empty lists write an empty array", func(t *testing.T) {
		for _, params := range [][]*parameter.Parameter{nil, {}, {nil}} {
			b, err := parametersToJSON(params)
			require.NoError(t, err)
			assert.JSONEq(t, `[]`, string(b))
		}
	})

	t.Run("omits nil default and config", func(t *testing.T) {
		bare, err := parameter.New().ProjectID(projID).Name("bare").Type(parameter.TypeText).Build()
		require.NoError(t, err)

		b, err := parametersToJSON([]*parameter.Parameter{bare})
		require.NoError(t, err)

		var docs []map[string]any
		require.NoError(t, json.Unmarshal(b, &docs))
		require.Len(t, docs, 1)
		assert.NotContains(t, docs[0], "default_value")
		assert.NotContains(t, docs[0], "config")
	})

	t.Run("fails on a value JSON cannot encode", func(t *testing.T) {
		bad, err := parameter.New().ProjectID(projID).Name("bad").DefaultValue(make(chan int)).Build()
		require.NoError(t, err)

		_, err = parametersToJSON([]*parameter.Parameter{bad})
		assert.Error(t, err)
	})

	t.Run("round-trips through parametersFromJSON", func(t *testing.T) {
		b, err := parametersToJSON([]*parameter.Parameter{p})
		require.NoError(t, err)

		got, err := parametersFromJSON(b)
		require.NoError(t, err)
		require.Len(t, got, 1)
		assert.Equal(t, pid, got[0].ID())
		assert.Equal(t, projID, got[0].ProjectID())
		assert.Equal(t, "area", got[0].Name())
		assert.Equal(t, parameter.TypeChoice, got[0].Type())
		assert.Equal(t, 2, got[0].Index())
		assert.True(t, got[0].Required())
		assert.True(t, got[0].Public())
		assert.Equal(t, "tokyo", got[0].DefaultValue())
		assert.Equal(t, map[string]any{"choices": []any{"tokyo", "osaka"}}, got[0].Config())
		assert.True(t, created.Equal(got[0].CreatedAt()))
		assert.True(t, updated.Equal(got[0].UpdatedAt()))
	})
}

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

func TestParametersFromJSON_Empty(t *testing.T) {
	for _, in := range []string{`[]`, `null`} {
		params, err := parametersFromJSON([]byte(in))
		require.NoError(t, err, in)
		assert.Empty(t, params, in)
	}
}

func TestParametersFromJSON_Invalid(t *testing.T) {
	validParam := id.NewParameterID().String()
	validProject := id.NewProjectID().String()
	doc := func(paramID, projectID string) string {
		return `[{"id":"` + paramID + `","project_id":"` + projectID + `","name":"area","type":"TEXT"}]`
	}

	tests := []struct {
		name string
		in   string
	}{
		{name: "empty input", in: ``},
		{name: "malformed JSON", in: `[{"id":`},
		{name: "object instead of array", in: `{"id":"` + validParam + `"}`},
		{name: "array of non-objects", in: `["area"]`},
		{name: "wrong field type", in: `[{"id":"` + validParam + `","project_id":"` + validProject + `","index":"first"}]`},
		{name: "missing parameter id", in: `[{"project_id":"` + validProject + `","name":"area"}]`},
		{name: "invalid parameter id", in: doc("not-an-id", validProject)},
		{name: "missing project id", in: `[{"id":"` + validParam + `","name":"area"}]`},
		{name: "invalid project id", in: doc(validParam, "not-an-id")},
		{name: "invalid timestamp", in: `[{"id":"` + validParam + `","project_id":"` + validProject + `","created_at":"yesterday"}]`},
		{
			name: "one bad entry among good ones",
			in: `[{"id":"` + validParam + `","project_id":"` + validProject + `","name":"ok"},` +
				`{"id":"bad","project_id":"` + validProject + `","name":"broken"}]`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			params, err := parametersFromJSON([]byte(tt.in))
			assert.Error(t, err)
			assert.Nil(t, params)
		})
	}
}
