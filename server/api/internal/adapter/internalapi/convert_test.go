package internalapi

import (
	"context"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	accountsuser "github.com/reearth/reearth-accounts/server/pkg/user"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	flowv1 "github.com/reearth/reearth-proto/gen/go/flow/v1"
	"github.com/samber/lo"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestToProject(t *testing.T) {
	u := accountsuser.New().NewID().Name("u").Email("u@example.com").MustBuild()
	updated := time.Date(2026, 10, 1, 0, 0, 0, 0, time.UTC)
	p := project.New().NewID().Workspace(accountsid.NewWorkspaceID()).
		Name("roads").Description("desc").Alias("roads-2026").Visibility(project.VisibilityPrivate).
		Readme("# R").License("MIT").Topics([]string{"gis"}).IsArchived(true).
		StarredBy([]string{u.ID().String(), accountsid.NewUserID().String()}).
		UpdatedAt(updated).MustBuild()

	got := toProject(context.Background(), p)
	assert.Equal(t, p.ID().String(), got.GetId())
	assert.Equal(t, p.Workspace().String(), got.GetWorkspaceId())
	assert.Equal(t, "roads", got.GetName())
	assert.Equal(t, "roads-2026", got.GetAlias())
	assert.Equal(t, "desc", got.GetDescription())
	assert.Equal(t, flowv1.ProjectVisibility_PROJECT_VISIBILITY_PRIVATE, got.GetVisibility())
	assert.Equal(t, "# R", got.GetReadme())
	assert.Equal(t, "MIT", got.GetLicense())
	assert.Equal(t, []string{"gis"}, got.GetTopics())
	assert.Equal(t, int64(2), got.GetStarCount())
	assert.True(t, got.GetIsArchived())
	assert.Equal(t, updated, got.GetUpdatedAt().AsTime())
	assert.Equal(t, p.CreatedAt().UTC(), got.GetCreatedAt().AsTime())
	assert.False(t, got.GetStarred(), "no user, so nothing is starred")

	got = toProject(adapter.AttachUser(context.Background(), u), p)
	assert.True(t, got.GetStarred(), "starred by the user in the context")

	assert.Nil(t, toProject(context.Background(), nil))
	assert.Len(t, toProjects(context.Background(), []*project.Project{p, nil}), 1, "nil entries are dropped")
}

func TestVisibility(t *testing.T) {
	assert.Equal(t, flowv1.ProjectVisibility_PROJECT_VISIBILITY_PUBLIC, toVisibility(project.VisibilityPublic))
	assert.Equal(t, flowv1.ProjectVisibility_PROJECT_VISIBILITY_PRIVATE, toVisibility(project.VisibilityPrivate))
	assert.Equal(t, flowv1.ProjectVisibility_PROJECT_VISIBILITY_UNSPECIFIED, toVisibility("unlisted"))

	tests := []struct {
		want *project.Visibility
		in   flowv1.ProjectVisibility
		ok   bool
	}{
		{nil, flowv1.ProjectVisibility_PROJECT_VISIBILITY_UNSPECIFIED, true},
		{lo.ToPtr(project.VisibilityPublic), flowv1.ProjectVisibility_PROJECT_VISIBILITY_PUBLIC, true},
		{lo.ToPtr(project.VisibilityPrivate), flowv1.ProjectVisibility_PROJECT_VISIBILITY_PRIVATE, true},
		{nil, flowv1.ProjectVisibility(99), false},
	}
	for _, tt := range tests {
		t.Run(tt.in.String(), func(t *testing.T) {
			got, ok := fromVisibility(tt.in)
			assert.Equal(t, tt.ok, ok)
			assert.Equal(t, tt.want, got)
		})
	}
}

func TestToDeployment(t *testing.T) {
	pid := id.NewProjectID()
	d := deployment.New().NewID().Workspace(accountsid.NewWorkspaceID()).Project(&pid).
		Version("v2").Description("nightly").IsHead(true).WorkflowURL("https://example.com/w.yml").MustBuild()

	got := toDeployment(d)
	assert.Equal(t, d.ID().String(), got.GetId())
	assert.Equal(t, d.Workspace().String(), got.GetWorkspaceId())
	assert.Equal(t, pid.String(), got.GetProjectId())
	assert.Equal(t, "v2", got.GetVersion())
	assert.Equal(t, "nightly", got.GetDescription())
	assert.True(t, got.GetIsHead())
	assert.NotContains(t, got.String(), "w.yml", "the workflow URL is not exposed")

	standalone := deployment.New().NewID().Workspace(accountsid.NewWorkspaceID()).MustBuild()
	assert.Nil(t, toDeployment(standalone).ProjectId, "no project_id for a deployment without a project")

	assert.Nil(t, toDeployment(nil))
	assert.Len(t, toDeployments([]*deployment.Deployment{d, nil}), 1)
}

func TestToTrigger(t *testing.T) {
	last := time.Date(2026, 10, 2, 0, 0, 0, 0, time.UTC)
	timed := trigger.New().NewID().Workspace(accountsid.NewWorkspaceID()).Deployment(id.NewDeploymentID()).
		Description("daily").EventSource(trigger.EventSourceTypeTimeDriven).TimeInterval(trigger.TimeIntervalEveryDay).
		Enabled(true).LastTriggered(last).MustBuild()

	got := toTrigger(timed)
	assert.Equal(t, timed.ID().String(), got.GetId())
	assert.Equal(t, timed.Deployment().String(), got.GetDeploymentId())
	assert.Equal(t, flowv1.TriggerEventSource_TRIGGER_EVENT_SOURCE_TIME_DRIVEN, got.GetEventSource())
	assert.Equal(t, flowv1.TriggerTimeInterval_TRIGGER_TIME_INTERVAL_EVERY_DAY, got.GetTimeInterval())
	assert.True(t, got.GetEnabled())
	assert.Equal(t, "daily", got.GetDescription())
	assert.Equal(t, last, got.GetLastTriggeredAt().AsTime())

	api := trigger.New().NewID().Workspace(accountsid.NewWorkspaceID()).Deployment(id.NewDeploymentID()).
		Description("hook").EventSource(trigger.EventSourceTypeAPIDriven).AuthToken("s3cret-token").MustBuild()
	got = toTrigger(api)
	assert.Equal(t, flowv1.TriggerEventSource_TRIGGER_EVENT_SOURCE_API_DRIVEN, got.GetEventSource())
	assert.Equal(t, flowv1.TriggerTimeInterval_TRIGGER_TIME_INTERVAL_UNSPECIFIED, got.GetTimeInterval())
	assert.Nil(t, got.GetLastTriggeredAt(), "never triggered")
	assert.NotContains(t, got.String(), "s3cret-token", "the auth token is never exposed")

	intervals := map[trigger.TimeInterval]flowv1.TriggerTimeInterval{
		trigger.TimeIntervalEveryHour:  flowv1.TriggerTimeInterval_TRIGGER_TIME_INTERVAL_EVERY_HOUR,
		trigger.TimeIntervalEveryDay:   flowv1.TriggerTimeInterval_TRIGGER_TIME_INTERVAL_EVERY_DAY,
		trigger.TimeIntervalEveryWeek:  flowv1.TriggerTimeInterval_TRIGGER_TIME_INTERVAL_EVERY_WEEK,
		trigger.TimeIntervalEveryMonth: flowv1.TriggerTimeInterval_TRIGGER_TIME_INTERVAL_EVERY_MONTH,
		"EVERY_YEAR":                   flowv1.TriggerTimeInterval_TRIGGER_TIME_INTERVAL_UNSPECIFIED,
	}
	for in, want := range intervals {
		assert.Equal(t, want, toTimeInterval(in), in)
	}
	assert.Equal(t, flowv1.TriggerEventSource_TRIGGER_EVENT_SOURCE_UNSPECIFIED, toEventSource("WEBHOOK"))

	assert.Nil(t, toTrigger(nil))
	assert.Len(t, toTriggers([]*trigger.Trigger{timed, nil}), 1)
}

func TestToJob(t *testing.T) {
	did, pid := id.NewDeploymentID(), id.NewProjectID()
	started := time.Date(2026, 10, 3, 0, 0, 0, 0, time.UTC)
	completed := started.Add(time.Minute)
	j, err := job.New().NewID().Workspace(accountsid.NewWorkspaceID()).Deployment(&did).ProjectID(&pid).
		Status(job.StatusFailed).StartedAt(started).CompletedAt(&completed).Build()
	require.NoError(t, err)

	got := toJob(j)
	assert.Equal(t, j.ID().String(), got.GetId())
	assert.Equal(t, did.String(), got.GetDeploymentId())
	assert.Equal(t, pid.String(), got.GetProjectId())
	assert.Equal(t, flowv1.JobStatus_JOB_STATUS_FAILED, got.GetStatus())
	assert.Equal(t, started, got.GetStartedAt().AsTime())
	assert.Equal(t, completed, got.GetCompletedAt().AsTime())

	running, err := job.New().NewID().Workspace(accountsid.NewWorkspaceID()).Status(job.StatusRunning).StartedAt(started).Build()
	require.NoError(t, err)
	got = toJob(running)
	assert.Nil(t, got.DeploymentId)
	assert.Nil(t, got.ProjectId)
	assert.Nil(t, got.GetCompletedAt(), "still running")

	statuses := map[job.Status]flowv1.JobStatus{
		job.StatusPending:   flowv1.JobStatus_JOB_STATUS_PENDING,
		job.StatusRunning:   flowv1.JobStatus_JOB_STATUS_RUNNING,
		job.StatusCompleted: flowv1.JobStatus_JOB_STATUS_COMPLETED,
		job.StatusFailed:    flowv1.JobStatus_JOB_STATUS_FAILED,
		job.StatusCancelled: flowv1.JobStatus_JOB_STATUS_CANCELLED,
		"PAUSED":            flowv1.JobStatus_JOB_STATUS_UNSPECIFIED,
	}
	for in, want := range statuses {
		assert.Equal(t, want, toJobStatus(in), in)
	}

	assert.Nil(t, toJob(nil))
	assert.Len(t, toJobs([]*job.Job{j, nil}), 1)
}

func TestToPageInfo(t *testing.T) {
	got := toPageInfo(interfaces.NewPageBasedInfo(42, 3, 10), 3, 10)
	assert.Equal(t, int64(42), got.GetTotalCount())
	assert.Equal(t, int32(3), got.GetPage())
	assert.Equal(t, int32(10), got.GetPageSize())

	assert.Zero(t, toPageInfo(nil, 1, 20).GetTotalCount())
}
