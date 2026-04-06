package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
)

type ParameterDocument struct {
	CreatedAt    time.Time   `bson:"created_at"`
	UpdatedAt    time.Time   `bson:"updated_at"`
	DefaultValue interface{} `bson:"default_value"`
	Config       interface{} `bson:"config"`
	ID           string      `bson:"id"`
	Name         string      `bson:"name"`
	Project      string      `bson:"project"`
	Type         string      `bson:"type"`
	Index        int         `bson:"index"`
	Required     bool        `bson:"required"`
	Public       bool        `bson:"public"`
}

type ParameterConsumer = Consumer[*ParameterDocument, *parameter.Parameter]

func NewParameterConsumer() *ParameterConsumer {
	return NewConsumer[*ParameterDocument](func(a *parameter.Parameter) bool {
		return true // No filtering needed for parameters
	})
}

func NewParameter(p *parameter.Parameter) (*ParameterDocument, string) {
	id := p.ID().String()
	return &ParameterDocument{
		CreatedAt:    p.CreatedAt(),
		ID:           id,
		Index:        p.Index(),
		Name:         p.Name(),
		Project:      p.ProjectID().String(),
		Required:     p.Required(),
		Public:       p.Public(),
		Type:         string(p.Type()),
		UpdatedAt:    p.UpdatedAt(),
		DefaultValue: p.DefaultValue(),
		Config:       p.Config(),
	}, id
}

func NewParameters(params parameter.ParameterList) ([]interface{}, []string) {
	res := make([]interface{}, 0, len(params))
	ids := make([]string, 0, len(params))
	for _, p := range params {
		if p == nil {
			continue
		}
		r, id := NewParameter(p)
		res = append(res, r)
		ids = append(ids, id)
	}
	return res, ids
}

func (d *ParameterDocument) Model() (*parameter.Parameter, error) {
	if d == nil {
		return nil, nil
	}

	pid, err := id.ParameterIDFrom(d.ID)
	if err != nil {
		return nil, err
	}

	projID, err := id.ProjectIDFrom(d.Project)
	if err != nil {
		return nil, err
	}

	return parameter.New().
		ID(pid).
		ProjectID(projID).
		Name(d.Name).
		Type(parameter.Type(d.Type)).
		Required(d.Required).
		Public(d.Public).
		DefaultValue(d.DefaultValue).
		Config(d.Config).
		Index(d.Index).
		CreatedAt(d.CreatedAt).
		UpdatedAt(d.UpdatedAt).
		Build()
}

func ParametersFromDocs(docs []ParameterDocument) []*parameter.Parameter {
	if len(docs) == 0 {
		return nil
	}

	params := make([]*parameter.Parameter, 0, len(docs))
	for _, d := range docs {
		p, err := d.Model()
		if err != nil {
			return nil
		}
		params = append(params, p)
	}
	return params
}

func ParametersToDocs(params []*parameter.Parameter) ([]ParameterDocument, []string) {
	if len(params) == 0 {
		return nil, nil
	}

	docs := make([]ParameterDocument, 0, len(params))
	ids := make([]string, 0, len(params))
	for _, p := range params {
		if p == nil {
			continue
		}
		d, id := NewParameter(p)
		docs = append(docs, *d)
		ids = append(ids, id)
	}
	return docs, ids
}
