package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/rerror"
	"golang.org/x/exp/slices"
)

type DeploymentDocument struct {
	ID          string    `bson:"id"`
	ProjectID   *string   `bson:"projectid,omitempty"`
	WorkspaceID string    `bson:"workspaceid"`
	WorkflowURL string    `bson:"workflowurl"`
	Description string    `bson:"description"`
	Version     string    `bson:"version"`
	UpdatedAt   time.Time `bson:"updatedat"`
	HeadID      *string   `bson:"headid,omitempty"`
	IsHead      bool      `bson:"ishead"`
}

type DeploymentConsumer = Consumer[*DeploymentDocument, *deployment.Deployment]

func NewDeploymentConsumer(workspaces []id.WorkspaceID) *DeploymentConsumer {
	return NewConsumer[*DeploymentDocument, *deployment.Deployment](func(d *deployment.Deployment) bool {
		return workspaces == nil || slices.Contains(workspaces, d.Workspace())
	})
}

func NewDeployment(d *deployment.Deployment) (*DeploymentDocument, error) {
	if d == nil {
		return nil, rerror.ErrNotFound
	}

	did := d.ID().String()
	if did == "" {
		return nil, rerror.ErrNotFound
	}

	var pid *string
	if p := d.Project(); p != nil {
		ps := p.String()
		pid = &ps
	}

	var hid *string
	if h := d.HeadID(); h != nil {
		hs := h.String()
		hid = &hs
	}

	return &DeploymentDocument{
		ID:          did,
		ProjectID:   pid,
		WorkspaceID: d.Workspace().String(),
		WorkflowURL: d.WorkflowURL(),
		Description: d.Description(),
		Version:     d.Version(),
		UpdatedAt:   d.UpdatedAt(),
		HeadID:      hid,
		IsHead:      d.IsHead(),
	}, nil
}

func (d *DeploymentDocument) Model() (*deployment.Deployment, error) {
	did, err := id.DeploymentIDFrom(d.ID)
	if err != nil {
		return nil, err
	}
	wid, err := id.WorkspaceIDFrom(d.WorkspaceID)
	if err != nil {
		return nil, err
	}

	builder := deployment.New().
		ID(did).
		Workspace(wid).
		WorkflowURL(d.WorkflowURL).
		Description(d.Description).
		Version(d.Version).
		UpdatedAt(d.UpdatedAt).
		IsHead(d.IsHead)

	if d.ProjectID != nil {
		pid, err := id.ProjectIDFrom(*d.ProjectID)
		if err != nil {
			return nil, err
		}
		builder = builder.Project(&pid)
	}

	if d.HeadID != nil {
		hid, err := id.DeploymentIDFrom(*d.HeadID)
		if err != nil {
			return nil, err
		}
		builder = builder.HeadID(&hid)
	}

	return builder.Build()
}
