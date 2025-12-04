package interactor

import (
	"encoding/json"
	"fmt"
	"strconv"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/variable"
)

type VariablesMode int

const (
	ModeAPIDriven VariablesMode = iota
	ModeTimeDriven
)

func resolveVariables(
	mode VariablesMode,
	projectParams map[string]variable.Variable,
	triggerVars map[string]variable.Variable,
	requestVars map[string]variable.Variable,
) (map[string]variable.Variable, error) {
	out := map[string]variable.Variable{}

	apply := func(src map[string]variable.Variable) error {
		for k, v := range src {
			cur, ok := out[k]
			if !ok {
				out[k] = v
				continue
			}
			// In case of type mismatch, we adopt the type of the overriding variable (v)
			nv, ok := coerceValue(v.Value, v.Type)
			if !ok {
				return fmt.Errorf("type mismatch on key %q (have=%s want=%s)", k, cur.Type, v.Type)
			}
			out[k] = variable.Variable{Key: k, Type: v.Type, Value: nv}
		}
		return nil
	}

	switch mode {
	case ModeAPIDriven:
		// REST /run: request.with ← trigger.variables ← project.parameters
		if err := apply(projectParams); err != nil {
			return nil, err
		}
		if err := apply(triggerVars); err != nil {
			return nil, err
		}
		if err := apply(requestVars); err != nil {
			return nil, err
		}
	case ModeTimeDriven:
		// REST /execute-scheduled: trigger.variables ← project.parameters
		if err := apply(projectParams); err != nil {
			return nil, err
		}
		if err := apply(triggerVars); err != nil {
			return nil, err
		}
	}
	return out, nil
}

func coerceValue(v any, t parameter.Type) (any, bool) {
	switch t {
	case parameter.TypeText:
		return fmt.Sprintf("%v", v), true
	case parameter.TypeNumber:
		switch n := v.(type) {
		case float64, float32, int, int32, int64, uint, uint32, uint64:
			return n, true
		case string:
			if f, err := strconv.ParseFloat(n, 64); err == nil {
				return f, true
			}
			return nil, false
		default:
			return nil, false
		}
	case parameter.TypeYesNo:
		switch b := v.(type) {
		case bool:
			return b, true
		case string:
			if x, err := strconv.ParseBool(b); err == nil {
				return x, true
			}
			return nil, false
		default:
			return nil, false
		}
	case parameter.TypeDatetime:
		switch s := v.(type) {
		case time.Time:
			return s, true
		case string:
			if t, err := time.Parse(time.RFC3339, s); err == nil {
				return t, true
			}
			return nil, false
		default:
			return nil, false
		}
	case parameter.TypeArray, parameter.TypeChoice:
		if _, ok := v.([]any); ok {
			return v, true
		}
		if s, ok := v.(string); ok {
			var a any
			if json.Unmarshal([]byte(s), &a) == nil {
				return a, true
			}
		}
		return nil, false
	case parameter.TypeColor:
		if s, ok := v.(string); ok {
			return s, true
		}
		return nil, false
	default:
		return v, true
	}
}

func projectParametersToMap(pl *parameter.ParameterList) map[string]variable.Variable {
	if pl == nil || len(*pl) == 0 {
		return nil
	}
	out := map[string]variable.Variable{}
	for _, p := range *pl {
		val := p.DefaultValue()
		if val == nil {
			continue
		}
		out[p.Name()] = variable.Variable{
			Key:   p.Name(),
			Type:  p.Type(),
			Value: val,
		}
	}
	return out
}

func normalizeRequestVars(
	vars map[string]interface{},
	schema map[string]variable.Variable,
) map[string]variable.Variable {
	if len(vars) == 0 {
		return nil
	}

	out := map[string]variable.Variable{}
	for k, v := range vars {
		if v == nil {
			continue
		}

		// Prioritize schema
		if s, ok := schema[k]; ok {
			coerced, ok2 := coerceValue(v, s.Type)
			if !ok2 {
				// If type coercion fails, default to text
				out[k] = variable.Variable{
					Key:   k,
					Type:  parameter.TypeText,
					Value: fmt.Sprintf("%v", v),
				}
				continue
			}
			out[k] = variable.Variable{
				Key:   k,
				Type:  s.Type,
				Value: coerced,
			}
			continue
		}

		// If no schema, default to text
		out[k] = variable.Variable{
			Key:   k,
			Type:  parameter.TypeText,
			Value: fmt.Sprintf("%v", v),
		}
	}

	return out
}
