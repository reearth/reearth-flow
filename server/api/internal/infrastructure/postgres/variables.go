package postgres

import (
	"encoding/json"

	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/variable"
)

type variableJSON struct {
	Key   string `json:"key"`
	Type  string `json:"type"`
	Value any    `json:"value"`
}

func variablesToJSON(vars []variable.Variable) ([]byte, error) {
	if len(vars) == 0 {
		return nil, nil
	}
	out := make([]variableJSON, 0, len(vars))
	for _, v := range vars {
		out = append(out, variableJSON{Key: v.Key, Type: string(v.Type), Value: v.Value})
	}
	return json.Marshal(out)
}

func variablesFromJSON(b []byte) ([]variable.Variable, error) {
	if len(b) == 0 {
		return nil, nil
	}
	var docs []variableJSON
	if err := json.Unmarshal(b, &docs); err != nil {
		return nil, err
	}
	if len(docs) == 0 {
		return nil, nil
	}
	out := make([]variable.Variable, 0, len(docs))
	for _, d := range docs {
		out = append(out, variable.Variable{Key: d.Key, Type: parameter.Type(d.Type), Value: d.Value})
	}
	return out, nil
}
