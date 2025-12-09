package variable_test

import (
	"encoding/json"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/variable"
	"github.com/stretchr/testify/assert"
)

func TestSliceToMap(t *testing.T) {
	tests := []struct {
		name  string
		input []variable.Variable
		want  map[string]variable.Variable
	}{
		{
			name:  "empty slice",
			input: []variable.Variable{},
			want:  nil,
		},
		{
			name:  "nil slice",
			input: nil,
			want:  nil,
		},
		{
			name: "normal case",
			input: []variable.Variable{
				{Key: "key1", Type: parameter.TypeText, Value: "value1"},
				{Key: "key2", Type: parameter.TypeNumber, Value: 42},
			},
			want: map[string]variable.Variable{
				"key1": {Key: "key1", Type: parameter.TypeText, Value: "value1"},
				"key2": {Key: "key2", Type: parameter.TypeNumber, Value: 42},
			},
		},
		{
			name: "skip empty key",
			input: []variable.Variable{
				{Key: "", Type: parameter.TypeText, Value: "value1"},
				{Key: "key2", Type: parameter.TypeNumber, Value: 42},
			},
			want: map[string]variable.Variable{
				"key2": {Key: "key2", Type: parameter.TypeNumber, Value: 42},
			},
		},
		{
			name: "duplicate keys (last wins)",
			input: []variable.Variable{
				{Key: "key1", Type: parameter.TypeText, Value: "value1"},
				{Key: "key1", Type: parameter.TypeText, Value: "value2"},
			},
			want: map[string]variable.Variable{
				"key1": {Key: "key1", Type: parameter.TypeText, Value: "value2"},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := variable.SliceToMap(tt.input)
			assert.Equal(t, tt.want, got)
		})
	}
}

func TestMapToSlice(t *testing.T) {
	tests := []struct {
		name  string
		input map[string]variable.Variable
		want  []variable.Variable
	}{
		{
			name:  "empty map",
			input: map[string]variable.Variable{},
			want:  nil,
		},
		{
			name:  "nil map",
			input: nil,
			want:  nil,
		},
		{
			name: "normal case",
			input: map[string]variable.Variable{
				"key1": {Key: "key1", Type: parameter.TypeText, Value: "value1"},
				"key2": {Key: "key2", Type: parameter.TypeNumber, Value: 42},
			},
			want: []variable.Variable{
				{Key: "key1", Type: parameter.TypeText, Value: "value1"},
				{Key: "key2", Type: parameter.TypeNumber, Value: 42},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := variable.MapToSlice(tt.input)

			if tt.want == nil {
				assert.Nil(t, got)
			} else {
				assert.Len(t, got, len(tt.want))
				for _, want := range tt.want {
					assert.Contains(t, got, want)
				}
			}
		})
	}
}

func TestToWorkerMap(t *testing.T) {
	now := time.Date(2024, 1, 1, 12, 0, 0, 0, time.UTC)

	tests := []struct {
		name  string
		input map[string]variable.Variable
		want  map[string]string
	}{
		{
			name:  "empty map",
			input: map[string]variable.Variable{},
			want:  map[string]string{},
		},
		{
			name: "text type",
			input: map[string]variable.Variable{
				"text": {Key: "text", Type: parameter.TypeText, Value: "hello world"},
			},
			want: map[string]string{
				"text": "hello world",
			},
		},
		{
			name: "color type",
			input: map[string]variable.Variable{
				"color": {Key: "color", Type: parameter.TypeColor, Value: "#FF0000"},
			},
			want: map[string]string{
				"color": "#FF0000",
			},
		},
		{
			name: "number type",
			input: map[string]variable.Variable{
				"number": {Key: "number", Type: parameter.TypeNumber, Value: 42.5},
			},
			want: map[string]string{
				"number": "42.5",
			},
		},
		{
			name: "yes/no type",
			input: map[string]variable.Variable{
				"bool": {Key: "bool", Type: parameter.TypeYesNo, Value: true},
			},
			want: map[string]string{
				"bool": "true",
			},
		},
		{
			name: "datetime type",
			input: map[string]variable.Variable{
				"datetime": {Key: "datetime", Type: parameter.TypeDatetime, Value: now},
			},
			want: map[string]string{
				"datetime": now.Format(time.RFC3339),
			},
		},
		{
			name: "datetime type with string value",
			input: map[string]variable.Variable{
				"datetime": {Key: "datetime", Type: parameter.TypeDatetime, Value: "2024-01-01"},
			},
			want: map[string]string{
				"datetime": "2024-01-01",
			},
		},
		{
			name: "complex type (JSON)",
			input: map[string]variable.Variable{
				"complex": {
					Key:   "complex",
					Type:  parameter.Type("custom"),
					Value: map[string]interface{}{"foo": "bar", "num": 123},
				},
			},
			want: map[string]string{
				"complex": `{"foo":"bar","num":123}`,
			},
		},
		{
			name: "mixed types",
			input: map[string]variable.Variable{
				"text":   {Key: "text", Type: parameter.TypeText, Value: "hello"},
				"number": {Key: "number", Type: parameter.TypeNumber, Value: 42},
				"bool":   {Key: "bool", Type: parameter.TypeYesNo, Value: false},
			},
			want: map[string]string{
				"text":   "hello",
				"number": "42",
				"bool":   "false",
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := variable.ToWorkerMap(tt.input)
			if tt.name == "complex type (JSON)" {
				assert.Len(t, got, len(tt.want))
				for k, v := range got {
					var gotJSON, wantJSON interface{}
					if err := json.Unmarshal([]byte(v), &gotJSON); err != nil {
						t.Fatalf("failed to unmarshal got value for key %s: %v", k, err)
					}
					if err := json.Unmarshal([]byte(tt.want[k]), &wantJSON); err != nil {
						t.Fatalf("failed to unmarshal want value for key %s: %v", k, err)
					}
					assert.Equal(t, wantJSON, gotJSON)
				}
			} else {
				assert.Equal(t, tt.want, got)
			}
		})
	}
}

func TestToString(t *testing.T) {
	tests := []struct {
		name     string
		variable variable.Variable
		want     string
	}{
		{
			name: "nil value",
			variable: variable.Variable{
				Key:   "nil",
				Type:  parameter.TypeText,
				Value: nil,
			},
			want: "<nil>",
		},
		{
			name: "array value",
			variable: variable.Variable{
				Key:   "array",
				Type:  parameter.Type("array"),
				Value: []string{"a", "b", "c"},
			},
			want: `["a","b","c"]`,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			m := map[string]variable.Variable{
				tt.variable.Key: tt.variable,
			}
			got := variable.ToWorkerMap(m)
			assert.Equal(t, tt.want, got[tt.variable.Key])
		})
	}
}
