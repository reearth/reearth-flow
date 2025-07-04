package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/pkg/cms"
	"github.com/reearth/reearthx/log"
)

func (r *mutationResolver) CreateCMSProject(ctx context.Context, input gqlmodel.CreateCMSProjectInput) (*gqlmodel.CMSProjectPayload, error) {
	cmsInput := cms.CreateProjectInput{
		WorkspaceID: string(input.WorkspaceID),
		Name:        input.Name,
		Alias:       input.Alias,
		Description: input.Description,
		License:     input.License,
		Readme:      input.Readme,
		Visibility:  convertGQLVisibilityToCMS(input.Visibility),
	}

	project, err := usecases(ctx).CMS.CreateCMSProject(ctx, cmsInput)
	if err != nil {
		log.Errorfc(ctx, "failed to create CMS project: %v", err)
		return nil, err
	}

	return &gqlmodel.CMSProjectPayload{
		Project: gqlmodel.CMSProjectFrom(project),
	}, nil
}

func (r *mutationResolver) UpdateCMSProject(ctx context.Context, input gqlmodel.UpdateCMSProjectInput) (*gqlmodel.CMSProjectPayload, error) {
	cmsInput := cms.UpdateProjectInput{
		ProjectID:   string(input.ProjectID),
		Name:        input.Name,
		Description: input.Description,
		License:     input.License,
		Readme:      input.Readme,
		Alias:       input.Alias,
	}

	if input.Visibility != nil {
		visibility := convertGQLVisibilityToCMS(*input.Visibility)
		cmsInput.Visibility = &visibility
	}

	project, err := usecases(ctx).CMS.UpdateCMSProject(ctx, cmsInput)
	if err != nil {
		log.Errorfc(ctx, "failed to update CMS project: %v", err)
		return nil, err
	}

	return &gqlmodel.CMSProjectPayload{
		Project: gqlmodel.CMSProjectFrom(project),
	}, nil
}

func (r *mutationResolver) DeleteCMSProject(ctx context.Context, input gqlmodel.DeleteCMSProjectInput) (*gqlmodel.DeleteCMSProjectPayload, error) {
	cmsInput := cms.DeleteProjectInput{
		ProjectID: string(input.ProjectID),
	}

	output, err := usecases(ctx).CMS.DeleteCMSProject(ctx, cmsInput)
	if err != nil {
		log.Errorfc(ctx, "failed to delete CMS project: %v", err)
		return nil, err
	}

	return &gqlmodel.DeleteCMSProjectPayload{
		ProjectID: gqlmodel.ID(output.ProjectID),
	}, nil
}

func (r *mutationResolver) CheckCMSAliasAvailability(ctx context.Context, input gqlmodel.CheckCMSAliasAvailabilityInput) (*gqlmodel.CheckCMSAliasAvailabilityPayload, error) {
	cmsInput := cms.CheckAliasAvailabilityInput{
		Alias: input.Alias,
	}

	output, err := usecases(ctx).CMS.CheckCMSAliasAvailability(ctx, cmsInput)
	if err != nil {
		log.Errorfc(ctx, "failed to check CMS alias availability: %v", err)
		return nil, err
	}

	return &gqlmodel.CheckCMSAliasAvailabilityPayload{
		Available: output.Available,
	}, nil
}

func convertGQLVisibilityToCMS(v gqlmodel.CMSVisibility) cms.Visibility {
	switch v {
	case gqlmodel.CMSVisibilityPublic:
		return cms.VisibilityPublic
	case gqlmodel.CMSVisibilityPrivate:
		return cms.VisibilityPrivate
	default:
		return cms.VisibilityPrivate
	}
}
