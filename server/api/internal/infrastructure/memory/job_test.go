package memory

import (
	"testing"

	"github.com/reearth/reearth-flow/api/internal/testutil/repotest"
)

func TestJob_CountRunningByWorkspace(t *testing.T) {
	repotest.JobCountRunning(t, NewJob())
}

func TestJob_SaveIfRunningBelow(t *testing.T) {
	repotest.JobSaveIfRunningBelow(t, NewJob())
}
