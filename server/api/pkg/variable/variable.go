package variable

import (
	"encoding/json"
	"fmt"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/parameter"
)

type Variable struct {
	Key   string
	Type  parameter.Type
	Value any
}

func SliceToMap(vars []Variable) map[string]Variable {
	if len(vars) == 0 {
		return nil
	}
	m := make(map[string]Variable, len(vars))
	for _, v := range vars {
		if v.Key == "" {
			continue
		}
		m[v.Key] = v
	}
	return m
}

func MapToSlice(m map[string]Variable) []Variable {
	if len(m) == 0 {
		return nil
	}
	out := make([]Variable, 0, len(m))
	for _, v := range m {
		out = append(out, v)
	}
	return out
}

func ToWorkerMap(vars map[string]Variable) map[string]string {
	out := make(map[string]string, len(vars))
	for k, v := range vars {
		out[k] = toString(v)
	}
	return out
}

func toString(v Variable) string {
	switch v.Type {
	case parameter.TypeText, parameter.TypeColor:
		return fmt.Sprintf("%v", v.Value)
	case parameter.TypeNumber, parameter.TypeYesNo:
		return fmt.Sprintf("%v", v.Value)
	case parameter.TypeDatetime:
		if t, ok := v.Value.(time.Time); ok {
			return t.Format(time.RFC3339)
		}
		return fmt.Sprintf("%v", v.Value)
	default:
		b, _ := json.Marshal(v.Value)
		return string(b)
	}
}
