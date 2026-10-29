package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/project"
)

func ToProject(p *project.Project) *Project {
	if p == nil {
		return nil
	}

	var sharedToken *string
	if p.SharedToken() != nil {
		sharedToken = p.SharedToken()
	}

	return &Project{
		ID:                IDFrom(p.ID()),
		Alias:             p.Alias(),
		CreatedAt:         p.CreatedAt(),
		CreatedByID:       IDFromRef(p.CreatedBy()),
		IsArchived:        p.IsArchived(),
		IsBasicAuthActive: p.IsBasicAuthActive(),
		IsLocked:          p.IsLocked(),
		BasicAuthUsername: p.BasicAuthUsername(),
		BasicAuthPassword: p.BasicAuthPassword(),
		License:           p.License(),
		Name:              p.Name(),
		Description:       p.Description(),
		Readme:            p.Readme(),
		SharedToken:       sharedToken,
		StarCount:         int(p.StarCount()),
		StarredBy:         p.StarredBy(),
		Topics:            p.Topics(),
		UpdatedAt:         p.UpdatedAt(),
		UpdatedByID:       IDFromRef(p.UpdatedBy()),
		Visibility:        ToProjectVisibility(p.Visibility()),
		WorkspaceID:       IDFrom(p.Workspace()),
	}
}

func ToProjectVisibility(v project.Visibility) ProjectVisibility {
	if v == project.VisibilityPublic {
		return ProjectVisibilityPublic
	}
	return ProjectVisibilityPrivate
}

func FromProjectVisibility(v *ProjectVisibility) *project.Visibility {
	if v == nil {
		return nil
	}
	res := project.VisibilityPrivate
	if *v == ProjectVisibilityPublic {
		res = project.VisibilityPublic
	}
	return &res
}
