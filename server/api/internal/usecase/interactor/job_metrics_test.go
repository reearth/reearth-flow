package interactor

import (
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/job/monitor"
	"github.com/stretchr/testify/assert"
)

func TestJob_MonitoredJobCount(t *testing.T) {
	j := &Job{monitor: monitor.NewMonitor()}
	assert.Equal(t, 0, j.MonitoredJobCount())

	j.monitor.Register("job-1", &monitor.Config{Cancel: func() {}})
	j.monitor.Register("job-2", &monitor.Config{Cancel: func() {}})
	assert.Equal(t, 2, j.MonitoredJobCount())

	j.monitor.Remove("job-1")
	assert.Equal(t, 1, j.MonitoredJobCount())
}

func TestJob_ActivePollerCount(t *testing.T) {
	j := &Job{activeWatchers: map[string]bool{}}
	assert.Equal(t, 0, j.ActivePollerCount())

	j.activeWatchers["job-1"] = true
	j.activeWatchers["job-2"] = true
	assert.Equal(t, 2, j.ActivePollerCount())
}

func TestJob_ActivePollerCountNilMap(t *testing.T) {
	j := &Job{}
	assert.Equal(t, 0, j.ActivePollerCount())
}
