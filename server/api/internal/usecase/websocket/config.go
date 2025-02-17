package websocket

type Config struct {
	GrpcServerURL string `envconfig:"REEARTH_FLOW_WEBSOCKET_SERVER" default:"localhost:50051"`
}
