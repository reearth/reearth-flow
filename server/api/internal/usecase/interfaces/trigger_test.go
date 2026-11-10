package interfaces

import (
	"errors"
	"fmt"
	"testing"

	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
)

func TestScheduledRunRefusal(t *testing.T) {
	t.Parallel()

	tests := []struct {
		err   error
		name  string
		code  trigger.RefusalCode
		retry bool
	}{
		{name: "the trigger is gone", err: ErrTriggerNotFound, code: trigger.RefusalNotFound},
		{name: "the deployment is gone", err: rerror.ErrNotFound, code: trigger.RefusalNotFound},
		{name: "disabled", err: ErrTriggerDisabled, code: trigger.RefusalTriggerDisabled},
		{name: "not time-driven", err: ErrTriggerNotTimeDriven, code: trigger.RefusalTriggerNotTimeDriven},
		{name: "a required variable is empty", err: fmt.Errorf("%w: area", ErrRequiredVariableMissing), code: trigger.RefusalInvalidVariables},
		{name: "a value does not fit", err: fmt.Errorf("%w: area", ErrInvalidVariableValue), code: trigger.RefusalInvalidVariables},
		{name: "an unknown variable", err: fmt.Errorf("%w: area", ErrUnknownVariable), code: trigger.RefusalInvalidVariables},
		{name: "not allowed", err: ErrOperationDenied, code: trigger.RefusalNotAllowed},
		{name: "no time-driven trigger runs left today", err: &UsageLimitError{Err: ErrTimeDrivenTriggerRunsPerDayLimit}, code: trigger.RefusalTimeDrivenTriggerRunsDailyLimit},
		{name: "the submit failed", err: ErrJobCreationFailed, code: trigger.RefusalServerError, retry: true},
		{name: "any other error", err: errors.New("boom"), code: trigger.RefusalServerError, retry: true},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()

			code, retry := ScheduledRunRefusal(tc.err)

			assert.Equal(t, tc.code, code)
			assert.Equal(t, tc.retry, retry)
		})
	}
}
