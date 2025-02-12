package document

type Config struct {
	WebsocketServerURL string `envconfig:"DOCUMENT_WEBSOCKETSERVERURL" default:"http://localhost:8000"`
}
