package gqlmodel

import (
	"testing"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestToTriggerRefusal(t *testing.T) {
	t.Parallel()

	at := time.Date(2026, 11, 6, 0, 3, 0, 0, time.UTC)
	until := at.Add(2 * time.Hour)

	t.Run("no refusal", func(t *testing.T) {
		t.Parallel()

		assert.Nil(t, ToTriggerRefusal(nil))
	})

	t.Run("a refusal that is retried", func(t *testing.T) {
		t.Parallel()

		got := ToTriggerRefusal(&trigger.Refusal{At: at, Code: trigger.RefusalServerError, RetryUntil: &until})

		require.NotNil(t, got)
		assert.Equal(t, TriggerRefusalCodeServerError, got.Code)
		assert.Equal(t, at, got.RefusedAt)
		assert.Equal(t, &until, got.RetryUntil)
	})

	t.Run("a missing deployment", func(t *testing.T) {
		t.Parallel()

		got := ToTriggerRefusal(&trigger.Refusal{At: at, Code: trigger.RefusalNotFound})

		require.NotNil(t, got)
		assert.Equal(t, TriggerRefusalCodeDeploymentNotFound, got.Code)
		assert.Nil(t, got.RetryUntil)
	})

	t.Run("the day's time-driven trigger runs used up", func(t *testing.T) {
		t.Parallel()

		got := ToTriggerRefusal(&trigger.Refusal{At: at, Code: trigger.RefusalTimeDrivenTriggerRunsDailyLimit})

		require.NotNil(t, got)
		assert.Equal(t, TriggerRefusalCodeTimeDrivenTriggerRunsDailyLimit, got.Code)
	})

	// Recorded before scheduled runs stopped being refused for running jobs.
	t.Run("a stored concurrent job limit refusal", func(t *testing.T) {
		t.Parallel()

		got := ToTriggerRefusal(&trigger.Refusal{At: at, Code: trigger.RefusalConcurrentJobLimit, RetryUntil: &until})

		require.NotNil(t, got)
		assert.Equal(t, TriggerRefusalCodeConcurrentJobLimit, got.Code)
	})

	t.Run("a code the schema does not name", func(t *testing.T) {
		t.Parallel()

		assert.Nil(t, ToTriggerRefusal(&trigger.Refusal{At: at, Code: trigger.RefusalTriggerNotTimeDriven}))
	})
}
