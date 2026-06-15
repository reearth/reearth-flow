package postgres

import (
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/variable"
	"github.com/reearth/reearthx/pgxx"
)

type variableJSON struct {
	Key   string `json:"key"`
	Type  string `json:"type"`
	Value any    `json:"value"`
}

func variablesToJSON(vars []variable.Variable) ([]byte, error) {
	out := make([]variableJSON, 0, len(vars))
	for _, v := range vars {
		out = append(out, variableJSON{Key: v.Key, Type: string(v.Type), Value: v.Value})
	}
	return pgxx.MarshalJSONBSlice(out)
}

func variablesFromJSON(b []byte) ([]variable.Variable, error) {
	docs, err := pgxx.UnmarshalJSONBSlice[variableJSON](b)
	if err != nil {
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
