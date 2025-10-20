package asset

import (
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/stretchr/testify/assert"
)

func TestUpload_Upload(t *testing.T) {
	t.Parallel()
	wid := id.NewWorkspaceID()
	timeNow := time.Now()
	uploadWithData := &Upload{
		uuid:          "1",
		workspace:     wid,
		fileName:      "file.test",
		contentLength: int64(1),
		expiresAt:     timeNow,
	}

	assert.Equal(t, "1", uploadWithData.UUID())
	assert.Equal(t, wid, uploadWithData.Workspace())
	assert.Equal(t, "file.test", uploadWithData.FileName())
	assert.Equal(t, int64(1), uploadWithData.ContentLength())
	assert.Equal(t, false, uploadWithData.Expired(timeNow))
	assert.Equal(t, timeNow, uploadWithData.ExpiresAt())
}
