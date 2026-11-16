package trigger

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestRetryPolicy_Window(t *testing.T) {
	t.Parallel()

	t.Run("hourly", func(t *testing.T) {
		t.Parallel()

		// Six attempts of 3 minutes, and waits of 1, 2, 4, 8 and 16 minutes.
		assert.Equal(t, 49*time.Minute, TimeIntervalEveryHour.RetryPolicy().Window())
	})

	t.Run("daily", func(t *testing.T) {
		t.Parallel()

		// Six attempts of 3 minutes, and waits of 5, 10, 20, 40 and 60 minutes.
		assert.Equal(t, 153*time.Minute, TimeIntervalEveryDay.RetryPolicy().Window())
	})
}

// Every retry of a run has to end before the next run is due, or the
// scheduler may skip that run.
func TestRetryPolicy_EndsBeforeTheNextRun(t *testing.T) {
	t.Parallel()

	// The shortest gap between two runs of each interval.
	gaps := map[TimeInterval]time.Duration{
		TimeIntervalEveryHour:  time.Hour,
		TimeIntervalEveryDay:   24 * time.Hour,
		TimeIntervalEveryWeek:  7 * 24 * time.Hour,
		TimeIntervalEveryMonth: 28 * 24 * time.Hour,
	}

	for interval, gap := range gaps {
		t.Run(string(interval), func(t *testing.T) {
			t.Parallel()

			assert.Less(t, interval.RetryPolicy().Window(), gap)
		})
	}
}
