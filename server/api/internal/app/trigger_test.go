package app

import (
	"errors"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strconv"
	"testing"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearthx/rerror"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestTriggerError(t *testing.T) {
	t.Parallel()

	respond := func(t *testing.T, err error) *httptest.ResponseRecorder {
		t.Helper()
		rec := httptest.NewRecorder()
		c := echo.New().NewContext(httptest.NewRequest(http.MethodPost, "/api/triggers/x/run", nil), rec)
		require.NoError(t, triggerError(c, err))
		return rec
	}

	t.Run("the day's API trigger calls used up: 429 with Retry-After", func(t *testing.T) {
		t.Parallel()

		rec := respond(t, &interfaces.APITriggerLimitError{ResetAt: time.Now().Add(time.Hour)})

		assert.Equal(t, http.StatusTooManyRequests, rec.Code)
		retryAfter, err := strconv.Atoi(rec.Header().Get(echo.HeaderRetryAfter))
		require.NoError(t, err)
		assert.InDelta(t, 3600, retryAfter, 2)
		assert.Contains(t, rec.Body.String(), "does not allow more API trigger calls today")
		assert.Contains(t, rec.Body.String(), `"code":"api_trigger_daily_limit"`)
	})

	t.Run("the day's deployment runs used up: 429 with its own code and Retry-After", func(t *testing.T) {
		t.Parallel()

		rec := respond(t, &interfaces.UsageLimitError{Err: interfaces.ErrDeploymentRunsPerDayLimit, ResetAt: time.Now().Add(time.Hour)})

		assert.Equal(t, http.StatusTooManyRequests, rec.Code)
		retryAfter, err := strconv.Atoi(rec.Header().Get(echo.HeaderRetryAfter))
		require.NoError(t, err)
		assert.InDelta(t, 3600, retryAfter, 2)
		assert.Contains(t, rec.Body.String(), `"code":"deployment_runs_daily_limit"`)
	})

	t.Run("a disabled trigger: 409 with its own code and no Retry-After", func(t *testing.T) {
		t.Parallel()

		rec := respond(t, interfaces.ErrTriggerDisabled)

		assert.Equal(t, http.StatusConflict, rec.Code)
		assert.Empty(t, rec.Header().Get(echo.HeaderRetryAfter))
		assert.Contains(t, rec.Body.String(), `"code":"trigger_disabled"`)
	})

	for name, tc := range map[string]struct {
		err  error
		want int
	}{
		"a wrong token: 401":                {interfaces.ErrInvalidTriggerToken, http.StatusUnauthorized},
		"a wrapped wrong token: 401":        {fmt.Errorf("run: %w", interfaces.ErrInvalidTriggerToken), http.StatusUnauthorized},
		"a caller without permission: 403":  {interfaces.ErrOperationDenied, http.StatusForbidden},
		"no such trigger: 404":              {rerror.ErrNotFound, http.StatusNotFound},
		"no such deployment: 404":           {interfaces.ErrDeploymentNotFound, http.StatusNotFound},
		"a required variable missing: 400":  {interfaces.ErrRequiredVariableMissing, http.StatusBadRequest},
		"a variable of the wrong type: 400": {interfaces.ErrInvalidVariableValue, http.StatusBadRequest},
		"an unknown variable: 400":          {interfaces.ErrUnknownVariable, http.StatusBadRequest},
		"any other error: 500":              {errors.New("boom"), http.StatusInternalServerError},
	} {
		t.Run(name, func(t *testing.T) {
			t.Parallel()

			rec := respond(t, tc.err)

			assert.Equal(t, tc.want, rec.Code)
			assert.Contains(t, rec.Body.String(), tc.err.Error())
		})
	}
}

func TestRetryAfterSeconds(t *testing.T) {
	t.Parallel()

	now := time.Date(2026, 10, 20, 23, 59, 0, 0, time.UTC)

	assert.Equal(t, 60, retryAfterSeconds(now.Add(time.Minute), now))
	assert.Equal(t, 61, retryAfterSeconds(now.Add(time.Minute+time.Millisecond), now), "rounds up so a caller never retries early")
	assert.Equal(t, 1, retryAfterSeconds(now.Add(-time.Second), now), "never less than one second")
}

// A scheduled run that can never succeed answers 2xx, so Cloud Scheduler does
// not retry it; one that may succeed later answers outside 2xx and is retried.
func TestScheduledTriggerError(t *testing.T) {
	t.Parallel()

	respond := func(t *testing.T, err error) *httptest.ResponseRecorder {
		t.Helper()
		rec := httptest.NewRecorder()
		c := echo.New().NewContext(httptest.NewRequest(http.MethodPost, "/api/triggers/x/execute-scheduled", nil), rec)
		require.NoError(t, scheduledTriggerError(c, err))
		return rec
	}

	skipped := []struct {
		err  error
		name string
		code string
	}{
		{name: "the trigger is gone", err: interfaces.ErrTriggerNotFound, code: "not_found"},
		{name: "the deployment is gone", err: rerror.ErrNotFound, code: "not_found"},
		{name: "the trigger is disabled", err: interfaces.ErrTriggerDisabled, code: "trigger_disabled"},
		{name: "the trigger is not time-driven", err: interfaces.ErrTriggerNotTimeDriven, code: "trigger_not_time_driven"},
		{name: "a required variable has no value", err: fmt.Errorf("%w: area", interfaces.ErrRequiredVariableMissing), code: "invalid_variables"},
		{name: "a variable does not fit its type", err: fmt.Errorf("%w: area", interfaces.ErrInvalidVariableValue), code: "invalid_variables"},
		{name: "the run is not allowed", err: interfaces.ErrOperationDenied, code: "operation_denied"},
		{name: "the day's time-driven trigger runs used up", err: &interfaces.UsageLimitError{Err: interfaces.ErrTimeDrivenTriggerRunsPerDayLimit, ResetAt: time.Now().Add(time.Hour)}, code: "time_driven_trigger_runs_daily_limit"},
	}
	for _, tc := range skipped {
		t.Run(tc.name+": skipped with 200", func(t *testing.T) {
			t.Parallel()

			rec := respond(t, tc.err)

			assert.Equal(t, http.StatusOK, rec.Code)
			assert.Contains(t, rec.Body.String(), `"status":"skipped"`)
			assert.Contains(t, rec.Body.String(), `"code":"`+tc.code+`"`)
		})
	}

	t.Run("the run's job is still being started: 409, retried", func(t *testing.T) {
		t.Parallel()

		rec := respond(t, interfaces.ErrScheduledRunInProgress)

		assert.Equal(t, http.StatusConflict, rec.Code)
		assert.Contains(t, rec.Body.String(), `"code":"scheduled_run_in_progress"`)
	})

	t.Run("the job could not be submitted: 500, retried", func(t *testing.T) {
		t.Parallel()

		assert.Equal(t, http.StatusInternalServerError, respond(t, interfaces.ErrJobCreationFailed).Code)
	})

	t.Run("any other error: 500, retried", func(t *testing.T) {
		t.Parallel()

		assert.Equal(t, http.StatusInternalServerError, respond(t, errors.New("boom")).Code)
	})
}

func TestScheduleTime(t *testing.T) {
	t.Parallel()

	request := func(header string) *http.Request {
		r := httptest.NewRequest(http.MethodPost, "/api/triggers/x/execute-scheduled", nil)
		if header != "" {
			r.Header.Set("X-CloudScheduler-ScheduleTime", header)
		}
		return r
	}

	t.Run("the scheduler's time", func(t *testing.T) {
		t.Parallel()

		got := scheduleTime(request("2026-11-06T00:00:00Z"))

		require.NotNil(t, got)
		assert.True(t, got.Equal(time.Date(2026, 11, 6, 0, 0, 0, 0, time.UTC)))
	})

	t.Run("an offset is kept as the same instant", func(t *testing.T) {
		t.Parallel()

		got := scheduleTime(request("2026-11-06T09:00:00+09:00"))

		require.NotNil(t, got)
		assert.True(t, got.Equal(time.Date(2026, 11, 6, 0, 0, 0, 0, time.UTC)))
	})

	t.Run("no header", func(t *testing.T) {
		t.Parallel()

		assert.Nil(t, scheduleTime(request("")))
	})

	t.Run("not a time", func(t *testing.T) {
		t.Parallel()

		assert.Nil(t, scheduleTime(request("tomorrow")))
	})
}
