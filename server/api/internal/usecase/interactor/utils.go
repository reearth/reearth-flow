package interactor

import (
	"encoding/json"
	"errors"
	"fmt"
	"math/big"
	"sort"
	"strconv"
	"strings"
	"time"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
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
			_, ok := out[k]
			if !ok {
				out[k] = v
				continue
			}
			// In case of type mismatch, we adopt the type of the overriding variable (v)
			nv, ok := coerceValue(v.Value, v.Type)
			if !ok {
				return fmt.Errorf("failed to coerce value for key %q to type %s", k, v.Type)
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
		case float64, float32, int, int8, int16, int32, int64, uint, uint8, uint16, uint32, uint64:
			return n, true
		case json.Number:
			if beyondExactFloat(n) {
				return nil, false
			}
			if f, err := n.Float64(); err == nil {
				return f, true
			}
			return nil, false
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
			if s == "" {
				return s, true
			}
			if t, err := time.Parse(time.RFC3339, s); err == nil {
				return t, true
			}
			if _, err := time.Parse(time.DateOnly, s); err == nil {
				return s, true
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
			if t == parameter.TypeChoice {
				return s, true
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

// maxExactFloatInt is 2^53, the largest integer from which float64 keeps every
// integer exactly.
var maxExactFloatInt = new(big.Int).Lsh(big.NewInt(1), 53)

// beyondExactFloat reports an integer literal past 2^53, which float64 would
// round. The literal itself is checked, so integers past int64 count too.
func beyondExactFloat(n json.Number) bool {
	s := string(n)
	if strings.ContainsAny(s, ".eE") {
		return false
	}
	i, ok := new(big.Int).SetString(s, 10)
	return ok && i.CmpAbs(maxExactFloatInt) > 0
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

// ensureRequiredParameters refuses vars that leave a required parameter
// without a value, naming every one that is missing.
func ensureRequiredParameters(params []*parameter.Parameter, vars map[string]variable.Variable) error {
	var missing []string
	for _, p := range params {
		if p == nil || !p.Required() {
			continue
		}
		if v, ok := vars[p.Name()]; !ok || isEmptyValue(v.Value) {
			missing = append(missing, p.Name())
		}
	}
	if len(missing) == 0 {
		return nil
	}
	return fmt.Errorf("%w: %s", interfaces.ErrRequiredVariableMissing, strings.Join(missing, ", "))
}

// isEmptyValue reports a value the user has not filled in. The engine receives
// an empty default as null, so a blank string counts as no value too.
func isEmptyValue(v any) bool {
	switch x := v.(type) {
	case nil:
		return true
	case string:
		return strings.TrimSpace(x) == ""
	case []any:
		return len(x) == 0
	case []string:
		return len(x) == 0
	default:
		return false
	}
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

// applyRunOverrides lays a manual run's overrides over the deployment's
// defaults. Each override takes its variable's type, so a value the client
// sent under another type is coerced rather than trusted. A key the
// deployment does not define, or a value that does not fit its type, refuses
// the run, naming every offending key.
func applyRunOverrides(
	defs []*parameter.Parameter,
	overrides []variable.Variable,
) (map[string]variable.Variable, error) {
	out := projectParametersToMap(parameter.NewParameterList(defs))
	if len(overrides) == 0 {
		return out, nil
	}
	if out == nil {
		out = map[string]variable.Variable{}
	}

	types := make(map[string]parameter.Type, len(defs))
	for _, p := range defs {
		if p != nil {
			types[p.Name()] = p.Type()
		}
	}

	var unknown, invalid []string
	for _, o := range overrides {
		t, ok := types[o.Key]
		if !ok {
			unknown = append(unknown, o.Key)
			continue
		}
		v, ok := coerceValue(o.Value, t)
		if ok && t == parameter.TypeArray && v != nil {
			// A string is decoded as JSON, so "3" would pass as an array.
			_, ok = v.([]any)
		}
		if !ok {
			invalid = append(invalid, o.Key)
			continue
		}
		out[o.Key] = variable.Variable{Key: o.Key, Type: t, Value: v}
	}

	// One refusal naming every offending key, of both kinds.
	var errs []error
	if len(unknown) > 0 {
		errs = append(errs, fmt.Errorf("%w: %s", interfaces.ErrUnknownVariable, strings.Join(unknown, ", ")))
	}
	if len(invalid) > 0 {
		errs = append(errs, fmt.Errorf("%w: %s", interfaces.ErrInvalidVariableValue, strings.Join(invalid, ", ")))
	}
	if err := errors.Join(errs...); err != nil {
		return nil, err
	}
	return out, nil
}

// runParameters copies the deployment's variables with the values a run
// resolved, so the job keeps a record of what it ran with. A variable the run
// left without a value is kept with none. Values for keys the deployment does
// not define (a trigger's stale value, an API call's extra `with`) reach the
// worker too, so they follow in key order.
func runParameters(defs []*parameter.Parameter, final map[string]variable.Variable) []*parameter.Parameter {
	out := make([]*parameter.Parameter, 0, len(defs)+len(final))
	defined := make(map[string]struct{}, len(defs))
	for _, p := range defs {
		if p == nil {
			continue
		}
		defined[p.Name()] = struct{}{}
		var v any
		if fv, ok := final[p.Name()]; ok {
			v = fv.Value
		}
		out = append(out, p.WithValue(v))
	}

	extra := make([]string, 0, len(final))
	for k := range final {
		if _, ok := defined[k]; !ok {
			extra = append(extra, k)
		}
	}
	sort.Strings(extra)
	for i, k := range extra {
		fv := final[k]
		p, err := parameter.New().Name(k).Type(fv.Type).DefaultValue(fv.Value).
			Index(len(defs) + i).Build()
		if err != nil {
			continue
		}
		out = append(out, p)
	}

	if len(out) == 0 {
		return nil
	}
	return out
}
