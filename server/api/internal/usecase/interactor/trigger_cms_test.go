package interactor

import (
	"context"
	"strings"
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/internal/rbac"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func cmsOrigin() job.Origin {
	return job.Origin{CMSItem: &job.CMSItemOrigin{
		ProjectID: "p1", ModelID: "m1", ItemID: "i1", ConnectionID: "c1",
		ProjectName: "Wayfarer", ModelName: "Buildings", ItemTitle: "Shinjuku 3-chome",
	}}
}

// cmsTrigger saves a CMS trigger on d directly, skipping the checks Create makes.
func (f *paramFixture) cmsTrigger(t *testing.T, d *deployment.Deployment, enabled bool) *trigger.Trigger {
	t.Helper()
	tr := trigger.New().NewID().Workspace(f.ws).Deployment(d.ID()).Description("t").Enabled(enabled).
		EventSource(trigger.EventSourceTypeCMS).CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()
	require.NoError(t, f.repo.Trigger.Save(f.ctx, tr))
	return tr
}

func TestTrigger_ExecuteCMSTrigger(t *testing.T) {
	t.Parallel()

	t.Run("runs with the item's values and records the trigger and the item", func(t *testing.T) {
		t.Parallel()
		f := newParamFixture(t)
		d := f.deployment(t, f.param(t, "cityGmlPath", 0, true, nil), f.param(t, "lang", 1, false, "ja"))
		tr := f.cmsTrigger(t, d, true)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteCMSTrigger(f.ctx, interfaces.ExecuteCMSTriggerParam{
			TriggerID: tr.ID(),
			Variables: map[string]any{"cityGmlPath": "https://assets.example.com/bldg.gml"},
			Origin:    cmsOrigin(),
		})

		// The fixture's batch refuses every job once its variables are decided.
		assert.ErrorIs(t, err, interfaces.ErrJobCreationFailed)
		require.Len(t, f.batch.submitted, 1)
		assert.Equal(t, "https://assets.example.com/bldg.gml", f.batch.submitted[0]["cityGmlPath"])
		assert.Equal(t, "ja", f.batch.submitted[0]["lang"], "a variable CMS leaves out takes its default")

		j := f.recordedJob(t)
		require.NotNil(t, j.Trigger())
		assert.Equal(t, tr.ID(), *j.Trigger())
		want := cmsOrigin()
		assert.Equal(t, &want, j.Origin())
	})

	t.Run("refuses a trigger that is not a CMS trigger", func(t *testing.T) {
		t.Parallel()
		f := newParamFixture(t)
		tr := f.trigger(t, f.deployment(t))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteCMSTrigger(f.ctx, interfaces.ExecuteCMSTriggerParam{TriggerID: tr.ID(), Origin: cmsOrigin()})

		assert.ErrorIs(t, err, interfaces.ErrTriggerNotCMS)
		assert.Empty(t, f.batch.submitted)
	})

	t.Run("refuses a trigger that is turned off", func(t *testing.T) {
		t.Parallel()
		f := newParamFixture(t)
		tr := f.cmsTrigger(t, f.deployment(t), false)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteCMSTrigger(f.ctx, interfaces.ExecuteCMSTriggerParam{TriggerID: tr.ID(), Origin: cmsOrigin()})

		assert.ErrorIs(t, err, interfaces.ErrTriggerDisabled)
		assert.Empty(t, f.batch.submitted)
	})

	t.Run("refuses a run that leaves a required variable empty before the plan is asked", func(t *testing.T) {
		t.Parallel()
		f := newParamFixture(t)
		tr := f.cmsTrigger(t, f.deployment(t, f.param(t, "cityGmlPath", 0, true, nil)), true)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteCMSTrigger(f.ctx, interfaces.ExecuteCMSTriggerParam{TriggerID: tr.ID(), Origin: cmsOrigin()})

		assert.ErrorIs(t, err, interfaces.ErrRequiredVariableMissing)
		assert.Empty(t, f.plan.asked)
		assert.Empty(t, f.batch.submitted)
	})

	t.Run("refuses a variable the deployment does not have", func(t *testing.T) {
		t.Parallel()
		f := newParamFixture(t)
		tr := f.cmsTrigger(t, f.deployment(t, f.param(t, "lang", 0, false, "ja")), true)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteCMSTrigger(f.ctx, interfaces.ExecuteCMSTriggerParam{
			TriggerID: tr.ID(), Variables: map[string]any{"unknown": "x"}, Origin: cmsOrigin(),
		})

		assert.ErrorIs(t, err, interfaces.ErrUnknownVariable)
		assert.Empty(t, f.batch.submitted)
	})

	t.Run("needs the user to be allowed to run the workspace's jobs", func(t *testing.T) {
		t.Parallel()
		f := newParamFixture(t)
		tr := f.cmsTrigger(t, f.deployment(t), true)
		var asked []string
		perm := NewMockPermissionChecker(func(_ context.Context, resource, action string) (bool, error) {
			asked = append(asked, resource+":"+action)
			return false, nil
		})
		i := NewTrigger(f.repo, f.gw, NewJob(f.repo, f.gw, perm), perm)

		_, err := i.ExecuteCMSTrigger(f.ctx, interfaces.ExecuteCMSTriggerParam{TriggerID: tr.ID(), Origin: cmsOrigin()})

		assert.ErrorIs(t, err, interfaces.ErrOperationDenied)
		assert.Equal(t, []string{rbac.ResourceJob + ":" + rbac.ActionCreate}, asked)
		assert.Empty(t, f.batch.submitted)
	})
}

func TestTrigger_ExecuteCMSTrigger_PlanLimits(t *testing.T) {
	t.Parallel()

	t.Run("counts as a deployment run", func(t *testing.T) {
		t.Parallel()
		f := newPlanFixture(t)
		usage := &usageCounter{}
		f.gw.UsageCounter = usage
		tr := f.trigger(t, trigger.New().EventSource(trigger.EventSourceTypeCMS))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteCMSTrigger(f.ctx, interfaces.ExecuteCMSTriggerParam{TriggerID: tr.ID(), Origin: cmsOrigin()})

		assert.ErrorIs(t, err, errUploadFailed)
		require.Len(t, usage.keys, 1)
		assert.True(t, strings.HasPrefix(usage.keys[0], deploymentRunsPerDay.keyPrefix), usage.keys[0])
	})

	t.Run("a run over the day's deployment runs is refused", func(t *testing.T) {
		t.Parallel()
		f := newPlanFixture(t)
		f.gw.UsageCounter = usedUpCounter{}
		tr := f.trigger(t, trigger.New().EventSource(trigger.EventSourceTypeCMS))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteCMSTrigger(f.ctx, interfaces.ExecuteCMSTriggerParam{TriggerID: tr.ID(), Origin: cmsOrigin()})

		assert.ErrorIs(t, err, interfaces.ErrDeploymentRunsPerDayLimit)
	})
}

func TestTrigger_CMSTriggerSave(t *testing.T) {
	t.Parallel()

	t.Run("a CMS trigger is created without a token or schedule, even with a required variable empty", func(t *testing.T) {
		t.Parallel()
		f := newParamFixture(t)
		d := f.deployment(t, f.param(t, "cityGmlPath", 0, true, nil))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		tr, token, err := i.Create(f.ctx, interfaces.CreateTriggerParam{
			WorkspaceID: f.ws, DeploymentID: d.ID(), Description: "t", Enabled: true, EventSource: trigger.EventSourceTypeCMS,
		})

		require.NoError(t, err)
		assert.Empty(t, token)
		assert.Equal(t, trigger.EventSourceTypeCMS, tr.EventSource())
		assert.Nil(t, tr.TimeInterval())
		assert.Nil(t, tr.AuthTokenHint())
	})

	t.Run("an API trigger made a CMS trigger loses its token", func(t *testing.T) {
		t.Parallel()
		f := newParamFixture(t)
		d := f.deployment(t)
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)
		tr, token, err := i.Create(f.ctx, interfaces.CreateTriggerParam{
			WorkspaceID: f.ws, DeploymentID: d.ID(), Description: "t", Enabled: true, EventSource: trigger.EventSourceTypeAPIDriven,
		})
		require.NoError(t, err)
		require.NotEmpty(t, token)

		updated, newToken, err := i.Update(f.ctx, interfaces.UpdateTriggerParam{ID: tr.ID(), EventSource: trigger.EventSourceTypeCMS})

		require.NoError(t, err)
		assert.Empty(t, newToken)
		assert.Equal(t, trigger.EventSourceTypeCMS, updated.EventSource())
		assert.False(t, updated.VerifyAuthToken(token))
	})
}
