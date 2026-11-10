package gqlmodel

import (
	"testing"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestEventSourceType_CMS(t *testing.T) {
	t.Parallel()

	assert.Equal(t, EventSourceTypeCms, ToEventSourceType(trigger.EventSourceTypeCMS))
	assert.Equal(t, trigger.EventSourceTypeCMS, FromEventSourceType(EventSourceTypeCms))
}

func TestToJob_TriggerAndOrigin(t *testing.T) {
	t.Parallel()

	t.Run("a job CMS started carries its trigger and item", func(t *testing.T) {
		t.Parallel()
		tid := id.NewTriggerID()
		j := job.New().NewID().Workspace(accountsid.NewWorkspaceID()).Trigger(&tid).
			Origin(&job.Origin{CMSItem: &job.CMSItemOrigin{
				ProjectID: "p1", ModelID: "m1", ItemID: "i1", ConnectionID: "c1",
				ProjectName: "Wayfarer", ModelName: "Buildings", ItemTitle: "Shinjuku 3-chome",
			}}).MustBuild()

		got := ToJob(j)

		require.NotNil(t, got.TriggerID)
		assert.Equal(t, ID(tid.String()), *got.TriggerID)
		require.NotNil(t, got.Origin)
		assert.Equal(t, &CMSItemJobOrigin{
			ProjectID: "p1", ModelID: "m1", ItemID: "i1", ConnectionID: "c1",
			ProjectName: "Wayfarer", ModelName: "Buildings", ItemTitle: "Shinjuku 3-chome",
		}, got.Origin.CmsItem)
	})

	t.Run("a job started in Flow has neither", func(t *testing.T) {
		t.Parallel()

		got := ToJob(job.New().NewID().Workspace(accountsid.NewWorkspaceID()).MustBuild())

		assert.Nil(t, got.TriggerID)
		assert.Nil(t, got.Origin)
	})
}
