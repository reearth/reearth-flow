package job

import (
	"time"

	"testing"

	"github.com/stretchr/testify/assert"
)

func TestJob_UserFacingLogsURL(t *testing.T) {
	j := &Job{userFacingLogsURL: "https://example.com/logs"}
	assert.Equal(t, "https://example.com/logs", j.UserFacingLogsURL())
}

func TestJob_SetUserFacingLogsURL(t *testing.T) {
	j := &Job{}
	j.SetUserFacingLogsURL("https://example.com/logs")
	assert.Equal(t, "https://example.com/logs", j.userFacingLogsURL)
}

func TestJob_SetStatus_KeepsCompletedAt(t *testing.T) {
	t.Parallel()

	finishedAt := time.Date(2026, 1, 1, 0, 0, 0, 0, time.UTC)
	j := New().NewID().Status(StatusRunning).MustBuild()
	j.SetCompletedAt(&finishedAt)

	j.SetStatus(StatusCompleted)

	assert.Equal(t, finishedAt, *j.CompletedAt())
}

func TestJob_SetStatus_StampsCompletedAt(t *testing.T) {
	t.Parallel()

	j := New().NewID().Status(StatusRunning).MustBuild()

	j.SetStatus(StatusFailed)

	assert.NotNil(t, j.CompletedAt())
}

func TestJob_SetNotificationURL_EmptyIsNone(t *testing.T) {
	t.Parallel()

	empty := ""
	j := New().NewID().NotificationURL(&empty).MustBuild()

	assert.Nil(t, j.NotificationURL())
}
