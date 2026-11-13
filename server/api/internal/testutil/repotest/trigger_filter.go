package repotest

import (
	"context"
	"testing"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TriggerFindByEventSource checks that FindByWorkspace filters by event source.
func TriggerFindByEventSource(t *testing.T, newRepo func(*testing.T) repo.Trigger) {
	t.Helper()
	ctx := context.Background()

	t.Run("lists only the triggers with the event source", func(t *testing.T) {
		t.Parallel()
		r := newRepo(t)
		ws := accountsid.NewWorkspaceID()
		save := func(source trigger.EventSourceType) *trigger.Trigger {
			tr := trigger.New().NewID().Workspace(ws).Deployment(id.NewDeploymentID()).Description(string(source)).
				EventSource(source).Enabled(true).CreatedAt(time.Now()).UpdatedAt(time.Now()).MustBuild()
			require.NoError(t, r.Save(ctx, tr))
			return tr
		}
		cms := save(trigger.EventSourceTypeCMS)
		save(trigger.EventSourceTypeAPIDriven)
		source := trigger.EventSourceTypeCMS
		page := &interfaces.PaginationParam{Page: &interfaces.PageBasedPaginationParam{Page: 1, PageSize: 10}}

		got, info, err := r.FindByWorkspace(ctx, ws, page, interfaces.TriggerFilter{EventSource: &source})

		require.NoError(t, err)
		require.Len(t, got, 1)
		assert.Equal(t, cms.ID(), got[0].ID())
		assert.Equal(t, int64(1), info.TotalCount)

		all, _, err := r.FindByWorkspace(ctx, ws, nil, interfaces.TriggerFilter{})
		require.NoError(t, err)
		assert.Len(t, all, 2)
	})
}
