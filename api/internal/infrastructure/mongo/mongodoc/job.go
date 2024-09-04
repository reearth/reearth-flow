package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/account/accountdomain"
	"golang.org/x/exp/slices"
)

type JobDocument struct {
	ID           string
	DeploymentID string
	WorkspaceID  string
	GCPJobID     string
	Status       string
	StartedAt    time.Time
	CompletedAt  *time.Time
}

type JobConsumer = Consumer[*JobDocument, *job.Job]

func NewJobConsumer(workspaces []accountdomain.WorkspaceID) *JobConsumer {
	return NewConsumer[*JobDocument, *job.Job](func(j *job.Job) bool {
		return workspaces == nil || slices.Contains(workspaces, j.Workspace())
	})
}

func NewJob(j *job.Job) (*JobDocument, string) {
	jid := j.ID().String()

	return &JobDocument{
		ID:           jid,
		DeploymentID: j.Deployment().String(),
		WorkspaceID:  j.Workspace().String(),
		GCPJobID:     j.GCPJobID(),
		Status:       string(j.Status()),
		StartedAt:    j.StartedAt(),
		CompletedAt:  j.CompletedAt(),
	}, jid
}

func (d *JobDocument) Model() (*job.Job, error) {
	jid, err := id.JobIDFrom(d.ID)
	if err != nil {
		return nil, err
	}
	did, err := id.DeploymentIDFrom(d.DeploymentID)
	if err != nil {
		return nil, err
	}
	wid, err := accountdomain.WorkspaceIDFrom(d.WorkspaceID)
	if err != nil {
		return nil, err
	}

	j := job.New().
		ID(jid).
		Deployment(did).
		Workspace(wid).
		Status(job.Status(d.Status)).
		StartedAt(d.StartedAt)

	if d.CompletedAt != nil {
		j = j.CompletedAt(d.CompletedAt)
	}

	return j.Build()
}
