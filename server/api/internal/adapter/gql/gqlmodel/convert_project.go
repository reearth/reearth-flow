package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/project"
)

func ToProject(p *project.Project) *Project {
	if p == nil {
		return nil
	}

	var sharedURL *string
	if p.SharedURL() != nil {
		sharedURL = p.SharedURL()
	}

	return &Project{
		ID:                IDFrom(p.ID()),
		CreatedAt:         p.CreatedAt(),
		IsArchived:        p.IsArchived(),
		IsBasicAuthActive: p.IsBasicAuthActive(),
		BasicAuthUsername: p.BasicAuthUsername(),
		BasicAuthPassword: p.BasicAuthPassword(),
		Name:              p.Name(),
		Description:       p.Description(),
		SharedURL:         sharedURL,
		UpdatedAt:         p.UpdatedAt(),
		WorkspaceID:       IDFrom(p.Workspace()),
	}
}
