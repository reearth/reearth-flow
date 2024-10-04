package gateway

type Container struct {
	Authenticator Authenticator
	File          File
	Batch         Batch
	Workflow      Workflow
}
