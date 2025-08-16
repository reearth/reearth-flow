package job

import (
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
