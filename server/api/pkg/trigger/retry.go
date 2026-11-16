package trigger

import "time"

// AttemptDeadline is how long the scheduler waits for one call to answer
// before the call counts as failed.
const AttemptDeadline = 3 * time.Minute

// RetryPolicy is how the scheduler tries a refused scheduled run again. The
// scheduler ignores Retry-After and allows at most 5 retries, so the waits
// double from a start that suits the interval. Every retry ends well before
// the next run, which the scheduler may skip while a run is still retrying.
type RetryPolicy struct {
	MinBackoff   time.Duration
	MaxBackoff   time.Duration
	Count        int
	MaxDoublings int
}

// RetryPolicy is how a refused run of a trigger with this interval is retried.
func (i TimeInterval) RetryPolicy() RetryPolicy {
	if i == TimeIntervalEveryHour {
		// 1, 2, 4, 8 and 16 minutes: about half an hour in all.
		return RetryPolicy{Count: 5, MinBackoff: time.Minute, MaxBackoff: 16 * time.Minute, MaxDoublings: 4}
	}
	// 5, 10, 20, 40 and 60 minutes: about two and a quarter hours.
	return RetryPolicy{Count: 5, MinBackoff: 5 * time.Minute, MaxBackoff: time.Hour, MaxDoublings: 4}
}

// Window is the longest the attempts of one run can last from its scheduled
// time: every attempt running to its deadline, and every wait between them.
func (p RetryPolicy) Window() time.Duration {
	total := time.Duration(p.Count+1) * AttemptDeadline
	wait := p.MinBackoff
	for i := range p.Count {
		total += min(wait, p.MaxBackoff)
		if i < p.MaxDoublings {
			wait *= 2
		}
	}
	return total
}
