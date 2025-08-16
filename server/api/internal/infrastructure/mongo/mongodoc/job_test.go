package mongodoc

import (
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearthx/account/accountdomain"
	"github.com/stretchr/testify/assert"
)

func TestJobDocument_UserFacingLogsURL(t *testing.T) {
	// Test conversion to/from MongoDB document
	j := job.New().
		ID(id.NewJobID()).
		Deployment(id.NewDeploymentID()).
		Workspace(accountdomain.NewWorkspaceID()).
		StartedAt(time.Now()).
		Status(job.StatusPending).
		UserFacingLogsURL("https://example.com/user-facing-logs").
		MustBuild()

	doc, _ := NewJob(j)
	assert.Equal(t, "https://example.com/user-facing-logs", doc.UserFacingLogsURL)

	modelJob, err := doc.Model()
	assert.NoError(t, err)
	assert.Equal(t, "https://example.com/user-facing-logs", modelJob.UserFacingLogsURL())
}