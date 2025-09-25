package gateway

type Container struct {
	File      File
	Batch     Batch
	Redis     Redis
	Scheduler Scheduler
	CMS       CMS
}
