package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
)

type ParameterDocument struct {
	CreatedAt time.Time   `bson:"created_at"`
	ID        string      `bson:"id"`
	Index     int         `bson:"index"`
	Name      string      `bson:"name"`
	Project   string      `bson:"project"`
	Required  bool        `bson:"required"`
	Type      string      `bson:"type"`
	UpdatedAt time.Time   `bson:"updated_at"`
	Value     interface{} `bson:"value"`
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
		CreatedAt: p.CreatedAt(),
		ID:        id,
		Index:     p.Index(),
		Name:      p.Name(),
		Project:   p.ProjectID().String(),
		Required:  p.Required(),
		Type:      string(p.Type()),
		UpdatedAt: p.UpdatedAt(),
		Value:     p.Value(),
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
		Value(d.Value).
		Index(d.Index).
		CreatedAt(d.CreatedAt).
		UpdatedAt(d.UpdatedAt).
		Build()
}
