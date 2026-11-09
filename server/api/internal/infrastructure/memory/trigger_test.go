package memory

import (
	"testing"

	"github.com/reearth/reearth-flow/api/internal/testutil/repotest"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
)

func TestTrigger_CountEnabledByWorkspace(t *testing.T) {
	t.Parallel()
	repotest.TriggerCountEnabled(t, NewTrigger())
}

func TestTrigger_SaveLastRefusal(t *testing.T) {
	t.Parallel()
	repotest.TriggerSaveLastRefusal(t, func(*testing.T) repo.Trigger { return NewTrigger() })
}
