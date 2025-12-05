package mongodoc

import (
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/variable"
)

type VariableDocument struct {
	Key   string `bson:"key"`
	Type  string `bson:"type"`
	Value any    `bson:"value"`
}

func VariablesToDoc(vars []variable.Variable) []VariableDocument {
	if len(vars) == 0 {
		return nil
	}

	vds := make([]VariableDocument, 0, len(vars))
	for _, v := range vars {
		vds = append(vds, VariableDocument{
			Key:   v.Key,
			Type:  string(v.Type),
			Value: v.Value,
		})
	}
	return vds
}

func VariablesFromDoc(docs []VariableDocument) []variable.Variable {
	if len(docs) == 0 {
		return nil
	}

	vars := make([]variable.Variable, 0, len(docs))
	for _, d := range docs {
		vars = append(vars, variable.Variable{
			Key:   d.Key,
			Type:  parameter.Type(d.Type),
			Value: d.Value,
		})
	}
	return vars
}
