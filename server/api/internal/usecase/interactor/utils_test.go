package interactor

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/variable"
	"github.com/reearth/reearthx/appx"
	"github.com/stretchr/testify/assert"
)

type mockPermissionChecker struct {
	checkPermissionFunc func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error)
}

func NewMockPermissionChecker(checkFunc func(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error)) *mockPermissionChecker {
	return &mockPermissionChecker{
		checkPermissionFunc: checkFunc,
	}
}

func (m *mockPermissionChecker) CheckPermission(ctx context.Context, authInfo *appx.AuthInfo, userId, resource, action string) (bool, error) {
	if m.checkPermissionFunc != nil {
		return m.checkPermissionFunc(ctx, authInfo, userId, resource, action)
	}
	return true, nil
}

func TestResolveVariables(t *testing.T) {
	pp := map[string]variable.Variable{
		"A": {Key: "A", Type: parameter.TypeText, Value: "pA"},
		"B": {Key: "B", Type: parameter.TypeText, Value: "pB"},
		"D": {Key: "D", Type: parameter.TypeText, Value: "pD"},
	}
	tv := map[string]variable.Variable{
		"C": {Key: "C", Type: parameter.TypeText, Value: "tC"},
		"D": {Key: "D", Type: parameter.TypeText, Value: "tD"},
	}
	rv := map[string]variable.Variable{
		"D": {Key: "D", Type: parameter.TypeText, Value: "rD"},
		"E": {Key: "E", Type: parameter.TypeText, Value: "rE"},
	}

	toStringMap := func(m map[string]variable.Variable) map[string]string {
		if m == nil {
			return nil
		}
		out := make(map[string]string, len(m))
		for k, v := range m {
			if s, ok := v.Value.(string); ok {
				out[k] = s
			}
		}
		return out
	}

	tests := []struct {
		name          string
		mode          VariablesMode
		projectParams map[string]variable.Variable
		triggerVars   map[string]variable.Variable
		requestVars   map[string]variable.Variable
		expected      map[string]string
	}{
		{
			name:          "ModeAPIDriven: Request Overrides Trigger Overrides Deployment",
			mode:          ModeAPIDriven,
			projectParams: pp,
			triggerVars:   tv,
			requestVars:   rv,
			expected: map[string]string{
				"A": "pA", // From PP
				"B": "pB", // From PP
				"C": "tC", // From TV
				"D": "rD", // From RV (overrides pD and tD)
				"E": "rE", // From RV
			},
		},
		{
			name:          "ModeTimeDriven: Trigger Overrides Deployment Overrides Project",
			mode:          ModeTimeDriven,
			projectParams: pp,
			triggerVars:   tv,
			requestVars:   nil,
			expected: map[string]string{
				"A": "pA", // From PP
				"B": "pB", // From PP
				"C": "tC", // From TV
				"D": "tD", // From TV (overrides pD)
			},
		},
		{
			name:          "Handles nil inputs",
			mode:          ModeAPIDriven,
			projectParams: nil,
			requestVars:   nil,
			expected:      map[string]string{},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			actual, err := resolveVariables(tc.mode, tc.projectParams, tc.triggerVars, tc.requestVars)
			assert.NoError(t, err)
			assert.Equal(t, tc.expected, toStringMap(actual))
		})
	}
}

func TestProjectParametersToMap(t *testing.T) {
	mockArray := []string{"item1", "item2"}
	mockTime := time.Date(2025, 1, 1, 10, 0, 0, 0, time.UTC)

	tests := []struct {
		name     string
		params   []map[string]interface{}
		expected map[string]variable.Variable
	}{
		{
			name: "Simple Types Conversion",
			params: []map[string]interface{}{
				{"name": "num", "type": parameter.TypeNumber, "defaultValue": 42.5},
				{"name": "bool", "type": parameter.TypeYesNo, "defaultValue": true},
				{"name": "text", "type": parameter.TypeText, "defaultValue": "hello"},
				{"name": "color", "type": parameter.TypeColor, "defaultValue": "#ff0000"},
			},
			expected: map[string]variable.Variable{
				"num": {
					Key:   "num",
					Type:  parameter.TypeNumber,
					Value: 42.5,
				},
				"bool": {
					Key:   "bool",
					Type:  parameter.TypeYesNo,
					Value: true,
				},
				"text": {
					Key:   "text",
					Type:  parameter.TypeText,
					Value: "hello",
				},
				"color": {
					Key:   "color",
					Type:  parameter.TypeColor,
					Value: "#ff0000",
				},
			},
		},
		{
			name: "Complex Types (JSON Marshal before, now raw value)",
			params: []map[string]interface{}{
				{"name": "arr", "type": parameter.TypeArray, "defaultValue": mockArray},
				{"name": "choice", "type": parameter.TypeChoice, "defaultValue": mockArray},
			},
			expected: map[string]variable.Variable{
				"arr": {
					Key:   "arr",
					Type:  parameter.TypeArray,
					Value: mockArray,
				},
				"choice": {
					Key:   "choice",
					Type:  parameter.TypeChoice,
					Value: mockArray,
				},
			},
		},
		{
			name: "Go Time Conversion (kept as time.Time)",
			params: []map[string]interface{}{
				{"name": "datetime", "type": parameter.TypeDatetime, "defaultValue": mockTime},
			},
			expected: map[string]variable.Variable{
				"datetime": {
					Key:   "datetime",
					Type:  parameter.TypeDatetime,
					Value: mockTime,
				},
			},
		},
		{
			name: "Nil and Empty String Handling",
			params: []map[string]interface{}{
				{"name": "emptyStr", "type": parameter.TypeText, "defaultValue": ""},
				{"name": "nilVal", "type": parameter.TypeText, "defaultValue": nil},
			},
			expected: map[string]variable.Variable{
				"emptyStr": {
					Key:   "emptyStr",
					Type:  parameter.TypeText,
					Value: "",
				},
			},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			pl := buildParameterList(t, tc.params)
			actual := projectParametersToMap(pl)

			assert.Equal(t, len(tc.expected), len(actual))
			for k, ev := range tc.expected {
				av, ok := actual[k]
				assert.True(t, ok, "key %s should exist", k)
				assert.Equal(t, ev.Type, av.Type, "type mismatch for key %s", k)
				assert.Equal(t, ev.Value, av.Value, "value mismatch for key %s", k)
			}
		})
	}
}

func buildParameterList(t *testing.T, defs []map[string]interface{}) *parameter.ParameterList {
	if len(defs) == 0 {
		return nil
	}

	list := make(parameter.ParameterList, len(defs))
	for i, def := range defs {
		builder := parameter.New()

		if name, ok := def["name"].(string); ok {
			builder.Name(name)
		} else {
			t.Fatalf("Missing 'name' field in parameter definition at index %d", i)
		}

		if typ, ok := def["type"].(parameter.Type); ok {
			builder.Type(typ)
		} else {
			t.Fatalf("Missing or invalid 'type' field in parameter definition for %s", def["name"])
		}

		if defaultValue, ok := def["defaultValue"]; ok {
			builder.DefaultValue(defaultValue)
		}

		param, err := builder.Build()
		assert.NoError(t, err)
		list[i] = param
	}
	return &list
}

func TestNormalizeRequestVars(t *testing.T) {
	arr := []interface{}{"a", "b"}

	tests := []struct {
		name     string
		input    map[string]interface{}
		schema   map[string]variable.Variable
		expected map[string]variable.Variable
	}{
		{
			name:     "nil input returns nil",
			input:    nil,
			schema:   nil,
			expected: nil,
		},
		{
			name:     "empty map returns nil",
			input:    map[string]interface{}{},
			schema:   nil,
			expected: nil,
		},
		{
			name: "no schema: everything becomes TEXT",
			input: map[string]interface{}{
				"s":  "str",
				"n":  10,
				"b":  true,
				"ar": arr,
			},
			schema: nil,
			expected: map[string]variable.Variable{
				"s": {
					Key:   "s",
					Type:  parameter.TypeText,
					Value: "str",
				},
				"n": {
					Key:   "n",
					Type:  parameter.TypeText,
					Value: "10",
				},
				"b": {
					Key:   "b",
					Type:  parameter.TypeText,
					Value: "true",
				},
				"ar": {
					Key:   "ar",
					Type:  parameter.TypeText,
					Value: fmt.Sprintf("%v", arr),
				},
			},
		},
		{
			name: "with schema: primitive types coerced by type",
			input: map[string]interface{}{
				"s":  "str",
				"n":  "1.5",
				"b":  "true",
				"tm": "2025-01-01T00:00:00Z",
			},
			schema: map[string]variable.Variable{
				"s":  {Key: "s", Type: parameter.TypeText},
				"n":  {Key: "n", Type: parameter.TypeNumber},
				"b":  {Key: "b", Type: parameter.TypeYesNo},
				"tm": {Key: "tm", Type: parameter.TypeDatetime},
			},
			expected: map[string]variable.Variable{
				"s": {
					Key:   "s",
					Type:  parameter.TypeText,
					Value: "str",
				},
				"n": {
					Key:   "n",
					Type:  parameter.TypeNumber,
					Value: 1.5,
				},
				"b": {
					Key:   "b",
					Type:  parameter.TypeYesNo,
					Value: true,
				},
				"tm": {
					Key:  "tm",
					Type: parameter.TypeDatetime,
				},
			},
		},
		{
			name: "with schema: array/choice coerced from JSON",
			input: map[string]interface{}{
				"arr": `["a","b"]`,
				"obj": `{"k":"v"}`,
			},
			schema: map[string]variable.Variable{
				"arr": {Key: "arr", Type: parameter.TypeArray},
				"obj": {Key: "obj", Type: parameter.TypeChoice},
			},
			expected: map[string]variable.Variable{
				"arr": {
					Key:  "arr",
					Type: parameter.TypeArray,
				},
				"obj": {
					Key:  "obj",
					Type: parameter.TypeChoice,
				},
			},
		},
		{
			name: "with schema: coercion failure falls back to TEXT",
			input: map[string]interface{}{
				"n": "not-a-number",
			},
			schema: map[string]variable.Variable{
				"n": {Key: "n", Type: parameter.TypeNumber},
			},
			expected: map[string]variable.Variable{
				"n": {
					Key:   "n",
					Type:  parameter.TypeText,
					Value: "not-a-number",
				},
			},
		},
		{
			name: "nil values are skipped",
			input: map[string]interface{}{
				"a": nil,
				"b": "value",
			},
			schema: nil,
			expected: map[string]variable.Variable{
				"b": {
					Key:   "b",
					Type:  parameter.TypeText,
					Value: "value",
				},
			},
		},
	}

	eqVarMap := func(expected, actual map[string]variable.Variable) {
		if expected == nil {
			assert.Nil(t, actual)
			return
		}
		assert.Equal(t, len(expected), len(actual))
		for k, ev := range expected {
			av, ok := actual[k]
			assert.True(t, ok, "key %s should exist", k)
			assert.Equal(t, ev.Type, av.Type, "type mismatch for key %s", k)

			// For special types like Datetime and Array, just check the type roughly
			switch ev.Type {
			case parameter.TypeDatetime:
				_, ok := av.Value.(time.Time)
				assert.True(t, ok, "value for key %s should be time.Time", k)
			case parameter.TypeArray, parameter.TypeChoice:
				// Since these come from JSON, avoid deep equal like []any or map[string]any
				assert.NotNil(t, av.Value, "value for key %s should not be nil", k)
			default:
				assert.Equal(t, ev.Value, av.Value, "value mismatch for key %s", k)
			}
		}
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			actual := normalizeRequestVars(tc.input, tc.schema)
			eqVarMap(tc.expected, actual)
		})
	}
}
