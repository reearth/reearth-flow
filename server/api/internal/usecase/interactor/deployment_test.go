package interactor

import (
	"context"
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/usecasex"
	"github.com/stretchr/testify/assert"
)

// nilDeploymentRepo is a stub that returns (nil, nil) from FindByID,
// reproducing the production scenario that caused the nil pointer panic.
type nilDeploymentRepo struct{}

func (r *nilDeploymentRepo) Filtered(_ repo.WorkspaceFilter) repo.Deployment { return r }
func (r *nilDeploymentRepo) FindByIDs(_ context.Context, _ id.DeploymentIDList) ([]*deployment.Deployment, error) {
	return nil, nil
}
func (r *nilDeploymentRepo) FindByID(_ context.Context, _ id.DeploymentID) (*deployment.Deployment, error) {
	return nil, nil
}
func (r *nilDeploymentRepo) FindByWorkspace(_ context.Context, _ accountsid.WorkspaceID, _ *interfaces.PaginationParam, _ *string) ([]*deployment.Deployment, *interfaces.PageBasedInfo, error) {
	return nil, nil, nil
}
func (r *nilDeploymentRepo) FindByProject(_ context.Context, _ id.ProjectID) (*deployment.Deployment, error) {
	return nil, nil
}
func (r *nilDeploymentRepo) FindByVersion(_ context.Context, _ accountsid.WorkspaceID, _ *id.ProjectID, _ string) (*deployment.Deployment, error) {
	return nil, nil
}
func (r *nilDeploymentRepo) FindHead(_ context.Context, _ accountsid.WorkspaceID, _ *id.ProjectID) (*deployment.Deployment, error) {
	return nil, nil
}
func (r *nilDeploymentRepo) FindVersions(_ context.Context, _ accountsid.WorkspaceID, _ *id.ProjectID) ([]*deployment.Deployment, error) {
	return nil, nil
}
func (r *nilDeploymentRepo) Save(_ context.Context, _ *deployment.Deployment) error { return nil }
func (r *nilDeploymentRepo) Remove(_ context.Context, _ id.DeploymentID) error      { return nil }

func TestDeployment_Execute_NilDeployment(t *testing.T) {
	checker := NewMockPermissionChecker(func(_ context.Context, _, _ string) (bool, error) {
		return true, nil
	})

	d := &Deployment{
		deploymentRepo:    &nilDeploymentRepo{},
		transaction:       &usecasex.NopTransaction{},
		permissionChecker: checker,
	}

	_, err := d.Execute(context.Background(), interfaces.ExecuteDeploymentParam{
		DeploymentID: id.NewDeploymentID(),
	})
	assert.ErrorContains(t, err, "deployment not found")
}
