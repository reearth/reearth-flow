package mongodoc

import (
	"log"
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
	log.Printf("DEBUG: Creating new job consumer with %d workspace filters", len(workspaces))
	if workspaces != nil {
		workspaceIDs := make([]string, 0, len(workspaces))
		for _, w := range workspaces {
			workspaceIDs = append(workspaceIDs, w.String())
		}
		log.Printf("DEBUG: Filtering jobs for workspaces: %v", workspaceIDs)
	} else {
		log.Printf("DEBUG: No workspace filtering applied to job consumer")
	}

	return NewConsumer[*JobDocument, *job.Job](func(j *job.Job) bool {
		result := workspaces == nil || slices.Contains(workspaces, j.Workspace())
		log.Printf("DEBUG: Job filter check for job %s in workspace %s: %v",
			j.ID().String(), j.Workspace().String(), result)
		return result
	})
}

func NewJob(j *job.Job) (*JobDocument, string) {
	if j == nil {
		log.Printf("ERROR: Attempted to create job document from nil job")
		return nil, ""
	}

	jid := j.ID().String()
	log.Printf("DEBUG: Creating job document for job ID %s", jid)

	edgeExecs := make([]EdgeExecutionDocument, 0, len(j.EdgeExecutions()))
	log.Printf("DEBUG: Converting %d edge executions", len(j.EdgeExecutions()))

	for i, e := range j.EdgeExecutions() {
		eid := e.ID()
		status := string(e.Status())

		var featureIDStr string
		if e.FeatureID() != nil {
			featureIDStr = *e.FeatureID()
		} else {
			featureIDStr = "<nil>"
		}

		log.Printf("DEBUG: Edge execution %d/%d: ID=%s, Status=%s, FeatureID=%s",
			i+1, len(j.EdgeExecutions()), eid, status, featureIDStr)

		edgeExecs = append(edgeExecs, EdgeExecutionDocument{
			ID:                  eid,
			Status:              status,
			StartedAt:           e.StartedAt(),
			CompletedAt:         e.CompletedAt(),
			FeatureID:           e.FeatureID(),
			IntermediateDataURL: e.IntermediateDataURL(),
		})
	}

	doc := &JobDocument{
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
	}

	log.Printf("DEBUG: Created job document with ID=%s, DeploymentID=%s, Status=%s, EdgeExecutions=%d",
		doc.ID, doc.DeploymentID, doc.Status, len(doc.EdgeExecutions))

	return doc, jid
}

func (d *JobDocument) Model() (*job.Job, error) {
	if d == nil {
		log.Printf("ERROR: Attempted to convert nil job document to model")
		return nil, nil
	}

	log.Printf("DEBUG: Converting job document to model: ID=%s, Status=%s", d.ID, d.Status)

	jid, err := id.JobIDFrom(d.ID)
	if err != nil {
		log.Printf("ERROR: Invalid job ID in document: %s, error: %v", d.ID, err)
		return nil, err
	}

	did, err := id.DeploymentIDFrom(d.DeploymentID)
	if err != nil {
		log.Printf("ERROR: Invalid deployment ID in document: %s, error: %v", d.DeploymentID, err)
		return nil, err
	}

	wid, err := accountdomain.WorkspaceIDFrom(d.WorkspaceID)
	if err != nil {
		log.Printf("ERROR: Invalid workspace ID in document: %s, error: %v", d.WorkspaceID, err)
		return nil, err
	}

	log.Printf("DEBUG: Converting %d edge execution documents to models", len(d.EdgeExecutions))
	edgeExecs := make([]*edge.EdgeExecution, 0, len(d.EdgeExecutions))

	for i, e := range d.EdgeExecutions {
		jobID, err := id.JobIDFrom(d.ID)
		if err != nil {
			log.Printf("ERROR: Failed to parse job ID %s for edge execution: %v", d.ID, err)
			return nil, err
		}

		var featureIDStr string
		if e.FeatureID != nil {
			featureIDStr = *e.FeatureID
		} else {
			featureIDStr = "<nil>"
		}

		log.Printf("DEBUG: Converting edge execution %d/%d: ID=%s, Status=%s, FeatureID=%s",
			i+1, len(d.EdgeExecutions), e.ID, e.Status, featureIDStr)

		edgeExec := edge.NewEdgeExecution(
			e.ID,
			jobID,
			d.DeploymentID,
			edge.Status(e.Status),
			e.StartedAt,
			e.CompletedAt,
			e.FeatureID,
			e.IntermediateDataURL,
		)

		edgeExecs = append(edgeExecs, edgeExec)
	}

	log.Printf("DEBUG: Building job model with ID=%s, Status=%s", jid.String(), d.Status)
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
		log.Printf("DEBUG: Job is completed at %s", d.CompletedAt.Format(time.RFC3339))
		j = j.CompletedAt(d.CompletedAt)
	} else {
		log.Printf("DEBUG: Job is not completed yet")
	}

	log.Printf("DEBUG: Building final job model")
	jobModel, err := j.Build()
	if err != nil {
		log.Printf("ERROR: Failed to build job model: %v", err)
		return nil, err
	}

	log.Printf("DEBUG: Successfully built job model with ID=%s, Status=%s, EdgeExecutions=%d",
		jobModel.ID().String(), jobModel.Status(), len(jobModel.EdgeExecutions()))
	return jobModel, nil
}
