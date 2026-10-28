package postgres

import (
	"encoding/json"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
)

// parameterJSON is the wire format for parameter snapshots stored in JSONB on
// jobs and deployments.
type parameterJSON struct {
	// Snapshots written before these were kept have none; the builder then
	// stamps them as it does any new parameter.
	CreatedAt    time.Time   `json:"created_at,omitzero"`
	UpdatedAt    time.Time   `json:"updated_at,omitzero"`
	DefaultValue interface{} `json:"default_value,omitempty"`
	Config       interface{} `json:"config,omitempty"`
	ID           string      `json:"id"`
	Name         string      `json:"name"`
	Type         string      `json:"type"`
	ProjectID    string      `json:"project_id"`
	Index        int         `json:"index"`
	Required     bool        `json:"required"`
	Public       bool        `json:"public"`
}

func parametersToJSON(params []*parameter.Parameter) ([]byte, error) {
	docs := make([]parameterJSON, 0, len(params))
	for _, param := range params {
		if param == nil {
			continue
		}
		docs = append(docs, parameterJSON{
			CreatedAt:    param.CreatedAt(),
			UpdatedAt:    param.UpdatedAt(),
			ID:           param.ID().String(),
			ProjectID:    param.ProjectID().String(),
			Name:         param.Name(),
			Type:         string(param.Type()),
			Index:        param.Index(),
			Required:     param.Required(),
			Public:       param.Public(),
			DefaultValue: param.DefaultValue(),
			Config:       param.Config(),
		})
	}
	return json.Marshal(docs)
}

func parametersFromJSON(b []byte) ([]*parameter.Parameter, error) {
	var docs []parameterJSON
	if err := json.Unmarshal(b, &docs); err != nil {
		return nil, err
	}
	params := make([]*parameter.Parameter, 0, len(docs))
	for _, doc := range docs {
		pid, err := id.ParameterIDFrom(doc.ID)
		if err != nil {
			return nil, err
		}
		projID, err := id.ProjectIDFrom(doc.ProjectID)
		if err != nil {
			return nil, err
		}
		p, err := parameter.New().
			ID(pid).
			ProjectID(projID).
			Name(doc.Name).
			Type(parameter.Type(doc.Type)).
			Index(doc.Index).
			Required(doc.Required).
			Public(doc.Public).
			DefaultValue(doc.DefaultValue).
			Config(doc.Config).
			CreatedAt(doc.CreatedAt).
			UpdatedAt(doc.UpdatedAt).
			Build()
		if err != nil {
			return nil, err
		}
		params = append(params, p)
	}
	return params, nil
}
