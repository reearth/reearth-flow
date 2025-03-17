package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/edge"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/account/accountdomain"
	"golang.org/x/exp/slices"
)

type JobDocument struct {
	ID             string                  `bson:"id"`
	Debug          *bool                   `bson:"debug"`
	DeploymentID   string                  `bson:"deploymentid"`
	WorkspaceID    string                  `bson:"workspaceid"`
	GCPJobID       string                  `bson:"gcpjobid"`
	LogsURL        string                  `bson:"logsurl"`
	Status         string                  `bson:"status"`
	StartedAt      time.Time               `bson:"startedat"`
	CompletedAt    *time.Time              `bson:"completedat"`
	MetadataURL    string                  `bson:"metadataurl"`
	OutputURLs     []string                `bson:"outputurls"`
	EdgeExecutions []EdgeExecutionDocument `bson:"edgeExecutions,omitempty"`
}

type EdgeExecutionDocument struct {
	ID                  string     `bson:"id"`
	Status              string     `bson:"status"`
	StartedAt           *time.Time `bson:"startedAt,omitempty"`
	CompletedAt         *time.Time `bson:"completedAt,omitempty"`
	FeatureID           *string    `bson:"featureId,omitempty"`
	IntermediateDataURL *string    `bson:"intermediateDataUrl,omitempty"`
}

type JobConsumer = Consumer[*JobDocument, *job.Job]

func NewJobConsumer(workspaces []accountdomain.WorkspaceID) *JobConsumer {
	return NewConsumer[*JobDocument, *job.Job](func(j *job.Job) bool {
		return workspaces == nil || slices.Contains(workspaces, j.Workspace())
	})
}

func NewJob(j *job.Job) (*JobDocument, string) {
	jid := j.ID().String()

	edgeExecs := make([]EdgeExecutionDocument, 0, len(j.EdgeExecutions()))
	for _, e := range j.EdgeExecutions() {
		eid := e.ID()
		edgeExecs = append(edgeExecs, EdgeExecutionDocument{
			ID:                  eid,
			Status:              string(e.Status()),
			StartedAt:           e.StartedAt(),
			CompletedAt:         e.CompletedAt(),
			FeatureID:           e.FeatureID(),
			IntermediateDataURL: e.IntermediateDataURL(),
		})
	}

	return &JobDocument{
		ID:             jid,
		Debug:          j.Debug(),
		DeploymentID:   j.Deployment().String(),
		WorkspaceID:    j.Workspace().String(),
		GCPJobID:       j.GCPJobID(),
		LogsURL:        j.LogsURL(),
		Status:         string(j.Status()),
		StartedAt:      j.StartedAt(),
		CompletedAt:    j.CompletedAt(),
		MetadataURL:    j.MetadataURL(),
		OutputURLs:     j.OutputURLs(),
		EdgeExecutions: edgeExecs,
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

	edgeExecs := make([]*edge.EdgeExecution, 0, len(d.EdgeExecutions))
	for _, e := range d.EdgeExecutions {
		jobID, err := id.JobIDFrom(d.ID)
		if err != nil {
			return nil, err
		}
		edgeExecs = append(edgeExecs, edge.NewEdgeExecution(
			e.ID,
			jobID,
			d.DeploymentID,
			edge.Status(e.Status),
			e.StartedAt,
			e.CompletedAt,
			e.FeatureID,
			e.IntermediateDataURL,
		))
	}

	j := job.New().
		ID(jid).
		Debug(d.Debug).
		Deployment(did).
		EdgeExecutions(edgeExecs).
		Workspace(wid).
		Status(job.Status(d.Status)).
		StartedAt(d.StartedAt).
		MetadataURL(d.MetadataURL).
		GCPJobID(d.GCPJobID).
		OutputURLs(d.OutputURLs).
		LogsURL(d.LogsURL)

	if d.CompletedAt != nil {
		j = j.CompletedAt(d.CompletedAt)
	}

	return j.Build()
}
