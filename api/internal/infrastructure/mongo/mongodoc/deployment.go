package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"golang.org/x/exp/slices"
)

type DeploymentDocument struct {
	ID          string    `bson:"id"`
	ProjectID   string    `bson:"projectid"`
	WorkspaceID string    `bson:"workspaceid"`
	WorkflowURL string    `bson:"workflowurl"`
	Version     string    `bson:"version"`
	CreatedAt   time.Time `bson:"createdat"`
	UpdatedAt   time.Time `bson:"updatedat"`
}

type DeploymentConsumer = Consumer[*DeploymentDocument, *deployment.Deployment]

func NewDeploymentConsumer(workspaces []accountdomain.WorkspaceID) *DeploymentConsumer {
	return NewConsumer[*DeploymentDocument, *deployment.Deployment](func(d *deployment.Deployment) bool {
		return workspaces == nil || slices.Contains(workspaces, d.Workspace())
	})
}

func NewDeployment(d *deployment.Deployment) (*DeploymentDocument, string) {
	did := d.ID().String()

	return &DeploymentDocument{
		ID:          did,
		ProjectID:   d.Project().String(),
		WorkspaceID: d.Workspace().String(),
		WorkflowURL: d.WorkflowUrl(),
		Version:     d.Version(),
		CreatedAt:   d.CreatedAt(),
		UpdatedAt:   d.UpdatedAt(),
	}, did
}

func (d *DeploymentDocument) Model() (*deployment.Deployment, error) {
	did, err := id.DeploymentIDFrom(d.ID)
	if err != nil {
		return nil, err
	}
	pid, err := id.ProjectIDFrom(d.ProjectID)
	if err != nil {
		return nil, err
	}
	wid, err := accountdomain.WorkspaceIDFrom(d.WorkspaceID)
	if err != nil {
		return nil, err
	}

	return deployment.New().
		ID(did).
		Project(pid).
		Workspace(wid).
		WorkflowURL(d.WorkflowURL).
		Version(d.Version).
		CreatedAt(d.CreatedAt).
		UpdatedAt(d.UpdatedAt).
		Build()
}
