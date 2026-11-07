package memory

import (
	"testing"

	"github.com/reearth/reearth-flow/api/internal/testutil/repotest"
)

func TestTrigger_CountEnabledByWorkspace(t *testing.T) {
	repotest.TriggerCountEnabled(t, NewTrigger())
}
