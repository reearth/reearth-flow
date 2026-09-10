package featureview

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestValidateFileIDAcceptsWhatTheEngineEmits(t *testing.T) {
	// `nodeID.port`, and `subgraphPrefix.nodeID.port` for a node inside a
	// subgraph — the two forms execution_dag.rs builds.
	for _, id := range []string{
		"7c9e6679-7425-40de-944b-e07fc1f90ae7.default",
		"3fa85f64-5717-4562-b3fc-2c963f66afa6.7c9e6679-7425-40de-944b-e07fc1f90ae7.rejected",
		"bldg_reader.success",
	} {
		assert.NoError(t, ValidateFileID(id), id)
	}
}

// The file id is client-supplied and lands in an object name on the WRITE side,
// where path.Clean on the read side is no protection.
func TestValidateFileIDRefusesPathTricks(t *testing.T) {
	for name, id := range map[string]string{
		"empty":            "",
		"separator":        "node.default/../../secret",
		"bare traversal":   "..",
		"leading dot":      ".hidden.default",
		"consecutive dots": "node..default",
		"backslash":        `node\default`,
		"nul byte":         "node\x00.default",
		"space":            "node id.default",
		"absolute":         "/artifacts/x",
	} {
		t.Run(name, func(t *testing.T) {
			assert.ErrorIs(t, ValidateFileID(id), ErrInvalidFileID)
		})
	}

	long := make([]byte, MaxFileIDLength+1)
	for i := range long {
		long[i] = 'a'
	}
	assert.ErrorIs(t, ValidateFileID(string(long)), ErrInvalidFileID)
}
