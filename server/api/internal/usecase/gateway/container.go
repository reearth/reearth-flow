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
	// worker is certain to have stopped it. An older job still marked pending
	// or running is stuck and no longer counts toward the plan's concurrent
	// jobs. 0 uses DefaultJobLifetime.
	JobLifetime time.Duration
}
