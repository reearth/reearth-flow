package gateway

type Container struct {
	Authenticator Authenticator
	File          File
	Batch         Batch
	LogRedis      Log
}
