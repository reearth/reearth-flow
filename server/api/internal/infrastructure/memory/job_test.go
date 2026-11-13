package memory

import (
	"testing"

	"github.com/reearth/reearth-flow/api/internal/testutil/repotest"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
)

func TestJob_CountRunningByWorkspace(t *testing.T) {
	repotest.JobCountRunning(t, NewJob())
}

func TestJob_SaveIfRunningBelow(t *testing.T) {
	repotest.JobSaveIfRunningBelow(t, NewJob())
}

func TestJob_ScheduledRun(t *testing.T) {
	t.Parallel()
	repotest.JobScheduledRun(t, func(*testing.T) repo.Job { return NewJob() })
}

func TestJob_TriggerAndOrigin(t *testing.T) {
	t.Parallel()
	repotest.JobTriggerAndOrigin(t, func(*testing.T) repo.Job { return NewJob() })
}

func TestJob_RunBy(t *testing.T) {
	t.Parallel()
	repotest.JobRunBy(t, func(*testing.T) repo.Job { return NewJob() })
}
