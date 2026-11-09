package interactor

import (
	"context"
	"testing"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/variable"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestReadWorkflowWith(t *testing.T) {
	t.Parallel()
	files := workflowFile{workflows: map[string]string{
		"editor.json": `{"id":"x","with":{"area":"tokyo","count":3,"ids":["a"],"note":null}}`,
		"hand.yml":    "id: x\nwith:\n  area: osaka\n  count: 2.5\n",
		"bare.yml":    "id: x\ngraphs: []\n",
		"empty.yml":   "",
		"broken.json": `{"with":`,
	}}

	tests := []struct {
		wantErr error
		want    map[string]any
		name    string
		url     string
	}{
		{
			name: "JSON from the editor",
			url:  "https://example.com/workflows/editor.json",
			want: map[string]any{"area": "tokyo", "count": 3, "ids": []any{"a"}, "note": nil},
		},
		{
			name: "YAML uploaded by hand",
			url:  "https://example.com/workflows/hand.yml",
			want: map[string]any{"area": "osaka", "count": 2.5},
		},
		{name: "a file with no with", url: "https://example.com/workflows/bare.yml"},
		{name: "an empty file", url: "https://example.com/workflows/empty.yml"},
		{name: "a bare file name", url: "editor.json", want: map[string]any{"area": "tokyo", "count": 3, "ids": []any{"a"}, "note": nil}},
		{name: "a missing file", url: "https://example.com/workflows/gone.json", wantErr: rerror.ErrNotFound},
		{name: "a URL with no file name", url: "https://example.com", wantErr: gateway.ErrInvalidFile},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			got, err := readWorkflowWith(context.Background(), files, tc.url)
			if tc.wantErr != nil {
				assert.ErrorIs(t, err, tc.wantErr)
				return
			}
			require.NoError(t, err)
			assert.Equal(t, tc.want, got)
		})
	}

	t.Run("a file that does not decode", func(t *testing.T) {
		t.Parallel()
		_, err := readWorkflowWith(context.Background(), files, "https://example.com/workflows/broken.json")
		assert.ErrorContains(t, err, "decode workflow broken.json")
	})
}

func TestWorkflowWithVariables(t *testing.T) {
	t.Parallel()
	param := func(name string, typ parameter.Type) *parameter.Parameter {
		p, err := parameter.New().Name(name).Type(typ).Build()
		require.NoError(t, err)
		return p
	}
	defs := []*parameter.Parameter{param("count", parameter.TypeNumber), nil, param("area", parameter.TypeText)}

	got := workflowWithVariables(defs, map[string]any{
		"count": 3,
		"area":  nil,
		"bad":   "x",
		"odd":   2,
	})

	assert.Equal(t, map[string]variable.Variable{
		"count": {Key: "count", Type: parameter.TypeNumber, Value: 3},
		"area":  {Key: "area", Type: parameter.TypeText},
		"bad":   {Key: "bad", Type: parameter.TypeText, Value: "x"},
		"odd":   {Key: "odd", Type: parameter.TypeText, Value: "2"},
	}, got)
	assert.Nil(t, workflowWithVariables(defs, nil))
}
