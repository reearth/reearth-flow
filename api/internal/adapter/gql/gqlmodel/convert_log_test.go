package gqlmodel

import (
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/log"
	"github.com/stretchr/testify/assert"
)

func TestToLog(t *testing.T) {
	t.Run("nil input", func(t *testing.T) {
		assert.Nil(t, ToLog(nil))
	})

	t.Run("success", func(t *testing.T) {
		jid := id.NewJobID()
		nid := id.NewNodeID()

		d := log.NewLog(jid, &nid, time.Now(), log.LevelInfo, "message")

		got := ToLog(d)
		assert.NotNil(t, got)
		assert.Equal(t, ID(jid.String()), got.JobID)
		assert.Equal(t, ID(nid.String()), *got.NodeID)
		assert.Equal(t, "message", got.Message)
		assert.Equal(t, LogLevel(log.LevelInfo), got.LogLevel)
		assert.False(t, got.Timestamp.IsZero())
	})
}
