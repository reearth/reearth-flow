package gateway

import "time"

type Container struct {
	File           File
	Batch          Batch
	Redis          Redis
	Scheduler      Scheduler
	CMS            CMS
	CloudRunWorker CloudRunWorker
	PolicyChecker  PolicyChecker
	UsageCounter   UsageCounter
	// JobLifetime is the longest a job can stay pending or running before the
	// worker is certain to have stopped it, after which it no longer counts
	// toward the plan's concurrent jobs. 0 uses DefaultJobLifetime.
	JobLifetime time.Duration
}
