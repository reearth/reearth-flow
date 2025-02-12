package document

import "github.com/reearth/reearthx/log"

type Config struct {
	WebsocketServerURL string `envconfig:"DOCUMENT_WEBSOCKETSERVERURL" default:"http://localhost:8000"`
}

var config = Config{
	WebsocketServerURL: "http://localhost:8000", // Default value
}

func SetConfig(cfg Config) {
	if cfg.WebsocketServerURL == "" {
		cfg.WebsocketServerURL = "http://localhost:8000"
		log.Warn("WebSocket server URL is empty, using default: http://localhost:8000")
	}
	log.Infof("Setting document config with WebSocket URL: %s", cfg.WebsocketServerURL)
	config = cfg
	if defaultClient != nil {
		defaultClient = NewClient(config.WebsocketServerURL)
	}
}

func GetConfig() Config {
	return config
}
