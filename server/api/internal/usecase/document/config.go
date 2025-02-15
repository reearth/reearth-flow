package document

type Config struct {
	GrpcServerURL string `envconfig:"DOCUMENT_GRPCSERVERURL" default:"localhost:50051"`
}
