package mongodoc

import (
	"testing"
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// TestNewDiagnosticDocument_NodeIDField_MatchesIDSegment pins the T5
// normalization fix: the nodeId bson field must carry the same
// nodeId-or-_job value as the document ID's segment, for both a real nodeId
// and an absent/empty one. Before the fix, only the ID got the "_job"
// fallback; the field stayed nil (absent nodeId) or the raw empty string
// (explicit empty nodeId), so a field-equality "_job" lookup could never
// match these rows.
func TestNewDiagnosticDocument_NodeIDField_MatchesIDSegment(t *testing.T) {
	t.Run("real nodeId", func(t *testing.T) {
		nodeID := "subgraph-a.node-4"
		event := &diagnostic.DiagnosticEvent{
			Schema:     diagnostic.DiagnosticSchemaV1,
			WorkflowID: "11111111-1111-1111-1111-111111111111",
			JobID:      "22222222-2222-2222-2222-222222222222",
			Timestamp:  time.Date(2026, 7, 16, 9, 31, 10, 0, time.UTC),
			WireDiagnostic: diagnostic.WireDiagnostic{
				Code:     "gltf.zero_face_solid",
				Category: "gltf",
				Severity: "warn",
				NodeID:   &nodeID,
				Message:  "solid has zero faces",
			},
		}

		doc := NewDiagnosticDocument(event)
		require.NotNil(t, doc.NodeID)
		assert.Equal(t, "subgraph-a.node-4", *doc.NodeID)
		assert.Equal(t, "22222222-2222-2222-2222-222222222222:subgraph-a.node-4:", doc.ID[:len("22222222-2222-2222-2222-222222222222:subgraph-a.node-4:")])
	})

	t.Run("absent nodeId falls back to the _job sentinel in both ID and field", func(t *testing.T) {
		event := &diagnostic.DiagnosticEvent{
			Schema:     diagnostic.DiagnosticSchemaV1,
			WorkflowID: "wf-123",
			JobID:      "job-456",
			Timestamp:  time.Now(),
			WireDiagnostic: diagnostic.WireDiagnostic{
				Code:     "internal.unclassified",
				Category: "internal",
				Severity: "warn",
				Message:  "job-level diagnostic without a nodeId",
			},
		}

		doc := NewDiagnosticDocument(event)
		require.NotNil(t, doc.NodeID)
		assert.Equal(t, JobDiagnosticNodeSegment, *doc.NodeID)
		assert.Equal(t, "job-456:_job:", doc.ID[:len("job-456:_job:")])
	})

	t.Run("explicit empty nodeId also falls back to the _job sentinel", func(t *testing.T) {
		emptyNodeID := ""
		event := &diagnostic.DiagnosticEvent{
			Schema:     diagnostic.DiagnosticSchemaV1,
			WorkflowID: "wf-123",
			JobID:      "job-456",
			Timestamp:  time.Now(),
			WireDiagnostic: diagnostic.WireDiagnostic{
				Code:     "internal.unclassified",
				Category: "internal",
				Severity: "warn",
				NodeID:   &emptyNodeID,
				Message:  "explicit empty nodeId also falls back",
			},
		}

		doc := NewDiagnosticDocument(event)
		require.NotNil(t, doc.NodeID)
		assert.Equal(t, JobDiagnosticNodeSegment, *doc.NodeID)
	})
}
