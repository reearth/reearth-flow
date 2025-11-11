package interactor

import (
	"context"
	"encoding/json"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/parameter"
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
	pp := map[string]string{"A": "pA", "B": "pB", "D": "pD"}
	dv := map[string]string{"B": "dB", "C": "dC"}
	tv := map[string]string{"C": "tC", "D": "tD"}
	rv := map[string]string{"D": "rD", "E": "rE"}

	tests := []struct {
		name           string
		mode           VariablesMode
		projectParams  map[string]string
		deploymentVars map[string]string
		triggerVars    map[string]string
		requestVars    map[string]string
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
			actual := resolveVariables(tc.mode, tc.projectParams, tc.deploymentVars, tc.triggerVars, tc.requestVars)
			assert.Equal(t, tc.expected, actual)
		})
	}
}

func TestProjectParametersToMap(t *testing.T) {
	mockArray := []string{"item1", "item2"}
	arrayJSONBytes, _ := json.Marshal(mockArray)
	mockTime := time.Date(2025, 1, 1, 10, 0, 0, 0, time.UTC)

	tests := []struct {
		name          string
		params        []map[string]interface{}
		expectedValue map[string]string
	}{
		{
			name: "Simple Types Conversion",
			params: []map[string]interface{}{
				{"name": "num", "type": parameter.TypeNumber, "defaultValue": 42.5},
				{"name": "bool", "type": parameter.TypeYesNo, "defaultValue": true},
				{"name": "text", "type": parameter.TypeText, "defaultValue": "hello"},
				{"name": "color", "type": parameter.TypeColor, "defaultValue": "#ff0000"},
			},
			expectedValue: map[string]string{
				"num":   "42.5",
				"bool":  "true",
				"text":  "hello",
				"color": "#ff0000",
			},
		},
		{
			name: "Complex Types (JSON Marshal)",
			params: []map[string]interface{}{
				{"name": "arr", "type": parameter.TypeArray, "defaultValue": mockArray},
				{"name": "choice", "type": parameter.TypeChoice, "defaultValue": mockArray},
			},
			expectedValue: map[string]string{
				"arr":    string(arrayJSONBytes),
				"choice": string(arrayJSONBytes),
			},
		},
		{
			name: "Go Time Conversion (Marshal)",
			params: []map[string]interface{}{
				{"name": "datetime", "type": parameter.TypeDatetime, "defaultValue": mockTime},
			},
			expectedValue: map[string]string{
				"datetime": "\"" + mockTime.Format(time.RFC3339Nano) + "\"",
			},
		},
		{
			name: "Nil and Empty String Handling",
			params: []map[string]interface{}{
				{"name": "emptyStr", "type": parameter.TypeText, "defaultValue": ""},
				{"name": "nilVal", "type": parameter.TypeText, "defaultValue": nil},
			},
			expectedValue: map[string]string{
				"emptyStr": "",
			},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			pl := buildParameterList(t, tc.params)
			actual := projectParametersToMap(pl)
			assert.Equal(t, tc.expectedValue, actual)
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

	arrJSON, _ := json.Marshal(arr)
	objJSON, _ := json.Marshal(obj)

	tests := []struct {
		name     string
		input    map[string]interface{}
		expected map[string]string
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
			expected: map[string]string{
				"s":   "str",
				"f64": "1.5",
				"f32": "2.5",
				"i":   "10",
				"i8":  "1",
				"i16": "2",
				"i32": "3",
				"i64": "4",
				"u":   "5",
				"u8":  "6",
				"u16": "7",
				"u32": "8",
				"u64": "9",
				"b":   "true",
			},
		},
		{
			name: "complex types marshalled as JSON",
			input: map[string]interface{}{
				"arr": arr,
				"obj": obj,
			},
			expected: map[string]string{
				"arr": string(arrJSON),
				"obj": string(objJSON),
			},
		},
		{
			name: "nil values are skipped",
			input: map[string]interface{}{
				"a": nil,
				"b": "value",
			},
			expected: map[string]string{
				"b": "value",
			},
		},
		{
			name:     "empty map returns nil",
			input:    map[string]interface{}{},
			expected: nil,
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			actual := normalizeRequestVars(tc.input)
			assert.Equal(t, tc.expected, actual)
		})
	}
}
