package interactor

import (
	"context"
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
	dv := map[string]variable.Variable{
		"B": {Key: "B", Type: parameter.TypeText, Value: "dB"},
		"C": {Key: "C", Type: parameter.TypeText, Value: "dC"},
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
		name           string
		mode           VariablesMode
		projectParams  map[string]variable.Variable
		deploymentVars map[string]variable.Variable
		triggerVars    map[string]variable.Variable
		requestVars    map[string]variable.Variable
		expected       map[string]string
	}{
		{
			name:           "ModeExecuteDeployment: Request Overrides Deployment Overrides Project",
			mode:           ModeExecuteDeployment,
			projectParams:  pp,
			deploymentVars: dv,
			requestVars:    rv,
			expected: map[string]string{
				"A": "pA", // From PP
				"B": "dB", // From DV (overrides pB)
				"C": "dC", // From DV
				"D": "rD", // From RV (overrides pD)
				"E": "rE", // From RV
			},
		},
		{
			name:           "ModeAPIDriven: Request Overrides Trigger Overrides Deployment",
			mode:           ModeAPIDriven,
			projectParams:  pp,
			deploymentVars: dv,
			triggerVars:    tv,
			requestVars:    rv,
			expected: map[string]string{
				"A": "pA", // From PP
				"B": "dB", // From DV (overrides pB)
				"C": "tC", // From TV (overrides dC)
				"D": "rD", // From RV (overrides pD and tD)
				"E": "rE", // From RV
			},
		},
		{
			name:           "ModeTimeDriven: Trigger Overrides Deployment Overrides Project",
			mode:           ModeTimeDriven,
			projectParams:  pp,
			deploymentVars: dv,
			triggerVars:    tv,
			requestVars:    rv, // Should be ignored
			expected: map[string]string{
				"A": "pA", // From PP
				"B": "dB", // From DV (overrides pB)
				"C": "tC", // From TV (overrides dC)
				"D": "tD", // From TV (overrides pD)
			},
		},
		{
			name:           "Handles nil inputs",
			mode:           ModeExecuteDeployment,
			projectParams:  nil,
			deploymentVars: dv,
			requestVars:    nil,
			expected: map[string]string{
				"B": "dB",
				"C": "dC",
			},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			actual, err := resolveVariables(tc.mode, tc.projectParams, tc.deploymentVars, tc.triggerVars, tc.requestVars)
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
	obj := map[string]interface{}{"k": "v"}

	tests := []struct {
		name     string
		input    map[string]interface{}
		expected map[string]variable.Variable
	}{
		{
			name:     "nil input returns nil",
			input:    nil,
			expected: nil,
		},
		{
			name: "primitive types",
			input: map[string]interface{}{
				"s":   "str",
				"f64": float64(1.5),
				"f32": float32(2.5),
				"i":   int(10),
				"i8":  int8(1),
				"i16": int16(2),
				"i32": int32(3),
				"i64": int64(4),
				"u":   uint(5),
				"u8":  uint8(6),
				"u16": uint16(7),
				"u32": uint32(8),
				"u64": uint64(9),
				"b":   true,
			},
			expected: map[string]variable.Variable{
				"s": {
					Key:   "s",
					Type:  parameter.TypeText,
					Value: "str",
				},
				"f64": {
					Key:   "f64",
					Type:  parameter.TypeNumber,
					Value: float64(1.5),
				},
				"f32": {
					Key:   "f32",
					Type:  parameter.TypeNumber,
					Value: float32(2.5),
				},
				"i": {
					Key:   "i",
					Type:  parameter.TypeNumber,
					Value: int(10),
				},
				"i8": {
					Key:   "i8",
					Type:  parameter.TypeNumber,
					Value: int8(1),
				},
				"i16": {
					Key:   "i16",
					Type:  parameter.TypeNumber,
					Value: int16(2),
				},
				"i32": {
					Key:   "i32",
					Type:  parameter.TypeNumber,
					Value: int32(3),
				},
				"i64": {
					Key:   "i64",
					Type:  parameter.TypeNumber,
					Value: int64(4),
				},
				"u": {
					Key:   "u",
					Type:  parameter.TypeNumber,
					Value: uint(5),
				},
				"u8": {
					Key:   "u8",
					Type:  parameter.TypeNumber,
					Value: uint8(6),
				},
				"u16": {
					Key:   "u16",
					Type:  parameter.TypeNumber,
					Value: uint16(7),
				},
				"u32": {
					Key:   "u32",
					Type:  parameter.TypeNumber,
					Value: uint32(8),
				},
				"u64": {
					Key:   "u64",
					Type:  parameter.TypeNumber,
					Value: uint64(9),
				},
				"b": {
					Key:   "b",
					Type:  parameter.TypeYesNo,
					Value: true,
				},
			},
		},
		{
			name: "complex types marshalled as ARRAY type",
			input: map[string]interface{}{
				"arr": arr,
				"obj": obj,
			},
			expected: map[string]variable.Variable{
				"arr": {
					Key:   "arr",
					Type:  parameter.TypeArray,
					Value: arr,
				},
				"obj": {
					Key:   "obj",
					Type:  parameter.TypeArray,
					Value: obj,
				},
			},
		},
		{
			name: "nil values are skipped",
			input: map[string]interface{}{
				"a": nil,
				"b": "value",
			},
			expected: map[string]variable.Variable{
				"b": {
					Key:   "b",
					Type:  parameter.TypeText,
					Value: "value",
				},
			},
		},
		{
			name:     "empty map returns nil",
			input:    map[string]interface{}{},
			expected: nil,
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
			assert.Equal(t, ev.Value, av.Value, "value mismatch for key %s", k)
		}
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			actual := normalizeRequestVars(tc.input)
			eqVarMap(tc.expected, actual)
		})
	}
}
