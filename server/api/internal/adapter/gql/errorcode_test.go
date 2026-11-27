package gql

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
)

// TestErrorCode_VersionNotRebuildable: the recovery view keys its message on
// this code, and the client wraps the sentinel with the server's detail.
func TestErrorCode_VersionNotRebuildable(t *testing.T) {
	t.Parallel()

	err := fmt.Errorf("%w: version 3 needs update 1", interfaces.ErrVersionNotRebuildable)
	assert.Equal(t, "VERSION_NOT_REBUILDABLE", ErrorCode(err))
}
