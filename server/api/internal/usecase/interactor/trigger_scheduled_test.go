package interactor

import (
	"testing"

	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/variable"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// A scheduled run that can never succeed returns an error the webhook can tell
// apart, so it answers without making the scheduler retry.
func TestTrigger_ExecuteTimeDrivenTrigger_PermanentErrors(t *testing.T) {
	t.Parallel()

	t.Run("a disabled trigger", func(t *testing.T) {
		t.Parallel()

		f := newParamFixture(t)
		tr := f.trigger(t, f.deployment(t))
		tr.SetEnabled(false)
		require.NoError(t, f.repo.Trigger.Save(f.ctx, tr))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})

		assert.ErrorIs(t, err, interfaces.ErrTriggerDisabled)
		assert.Empty(t, f.batch.submitted)
	})

	t.Run("a trigger that is not time-driven", func(t *testing.T) {
		t.Parallel()

		f := newParamFixture(t)
		tr := f.trigger(t, f.deployment(t))
		tr.SetEventSource("API_DRIVEN")
		tr.SetTimeInterval("")
		require.NoError(t, f.repo.Trigger.Save(f.ctx, tr))
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})

		assert.ErrorIs(t, err, interfaces.ErrTriggerNotTimeDriven)
		assert.Empty(t, f.batch.submitted)
	})

	t.Run("a trigger value that does not fit its variable's type", func(t *testing.T) {
		t.Parallel()

		f := newParamFixture(t)
		d := f.deployment(t, f.param(t, "area", 0, false, "deployed"))
		tr := f.trigger(t, d, variable.Variable{Key: "area", Type: parameter.TypeNumber, Value: "not a number"})
		i := NewTrigger(f.repo, f.gw, f.job, f.perm)

		_, err := i.ExecuteTimeDrivenTrigger(f.ctx, interfaces.ExecuteTimeDrivenTriggerParam{TriggerID: tr.ID()})

		assert.ErrorIs(t, err, interfaces.ErrInvalidVariableValue)
		assert.Empty(t, f.batch.submitted)
	})
}
