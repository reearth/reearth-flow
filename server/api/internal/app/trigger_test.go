package app

import (
	"errors"
	"net/http"
	"net/http/httptest"
	"strconv"
	"testing"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestTriggerError(t *testing.T) {
	respond := func(err error) *httptest.ResponseRecorder {
		rec := httptest.NewRecorder()
		c := echo.New().NewContext(httptest.NewRequest(http.MethodPost, "/api/triggers/x/run", nil), rec)
		require.NoError(t, triggerError(c, err))
		return rec
	}

	t.Run("the day's API trigger calls used up: 429 with Retry-After", func(t *testing.T) {
		rec := respond(&interfaces.APITriggerLimitError{ResetAt: time.Now().Add(time.Hour)})

		assert.Equal(t, http.StatusTooManyRequests, rec.Code)
		retryAfter, err := strconv.Atoi(rec.Header().Get(echo.HeaderRetryAfter))
		require.NoError(t, err)
		assert.InDelta(t, 3600, retryAfter, 2)
		assert.Contains(t, rec.Body.String(), "does not allow more API trigger calls today")
		assert.Contains(t, rec.Body.String(), `"code":"api_trigger_daily_limit"`)
	})

	t.Run("the plan's jobs all running: 429 with its own code and Retry-After", func(t *testing.T) {
		rec := respond(&interfaces.JobLimitError{})

		assert.Equal(t, http.StatusTooManyRequests, rec.Code)
		assert.Equal(t, "300", rec.Header().Get(echo.HeaderRetryAfter))
		assert.Contains(t, rec.Body.String(), `"code":"concurrent_job_limit"`)
	})

	t.Run("any other error: 500", func(t *testing.T) {
		assert.Equal(t, http.StatusInternalServerError, respond(errors.New("boom")).Code)
	})
}

func TestRetryAfterSeconds(t *testing.T) {
	now := time.Date(2026, 10, 20, 23, 59, 0, 0, time.UTC)

	assert.Equal(t, 60, retryAfterSeconds(now.Add(time.Minute), now))
	assert.Equal(t, 61, retryAfterSeconds(now.Add(time.Minute+time.Millisecond), now), "rounds up so a caller never retries early")
	assert.Equal(t, 1, retryAfterSeconds(now.Add(-time.Second), now), "never less than one second")
}
