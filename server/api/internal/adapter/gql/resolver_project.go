package gql

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/pkg/id"
)

func (r *Resolver) Project() ProjectResolver {
	return &projectResolver{r}
}

type projectResolver struct{ *Resolver }

func (r *projectResolver) Deployment(ctx context.Context, obj *gqlmodel.Project) (*gqlmodel.Deployment, error) {
	return loaders(ctx).Deployment.FindByProject(ctx, obj.ID)
}

func (r *projectResolver) Parameters(ctx context.Context, obj *gqlmodel.Project) ([]*gqlmodel.Parameter, error) {
	sid, err := gqlmodel.ToID[id.Project](obj.ID)
	if err != nil {
		return nil, err
	}

	fmt.Println("GOING TO GET PARAMETERS FOR PROJECT", sid)
	parameters, err := usecases(ctx).Parameter.FetchByProject(ctx, sid)
	if err != nil {
		return nil, err
	}

	res := gqlmodel.ToParameters(parameters)
	return res, nil
}

func (r *projectResolver) Workspace(ctx context.Context, obj *gqlmodel.Project) (*gqlmodel.Workspace, error) {
	return dataloaders(ctx).Workspace.Load(obj.WorkspaceID)
}
