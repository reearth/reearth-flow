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

func TestValidateFileIDAcceptsAuthorNamedPorts(t *testing.T) {
	for _, id := range []string{
		"7c9e6679-7425-40de-944b-e07fc1f90ae7.建物",
		"7c9e6679-7425-40de-944b-e07fc1f90ae7.high rise",
		"7c9e6679-7425-40de-944b-e07fc1f90ae7.a..b",
		"7c9e6679-7425-40de-944b-e07fc1f90ae7.50%",
	} {
		assert.NoError(t, ValidateFileID(id), id)
	}
}

func TestValidateFileIDRefusesPathTricks(t *testing.T) {
	for name, id := range map[string]string{
		"empty":          "",
		"separator":      "node.default/../../secret",
		"bare traversal": "..",
		"single dot":     ".",
		"leading dot":    ".hidden.default",
		"backslash":      `node\default`,
		"nul byte":       "node\x00.default",
		"newline":        "node.a\nb",
		"invalid utf-8":  "node.\xff",
		"absolute":       "/artifacts/x",
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
