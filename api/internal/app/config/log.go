package config

type RedisLogConfig struct {
	Addr     string `pp:",omitempty"`
	Password string `pp:",omitempty"`
	DB       int    `pp:",omitempty"`
}

func (r RedisLogConfig) IsConfigured() bool {
	return r.Addr != ""
}

type GCSLogConfig struct {
	BucketName              string `pp:",omitempty"`
	PublicationCacheControl string `pp:",omitempty"`
}

func (g GCSLogConfig) IsConfigured() bool {
	return g.BucketName != ""
}
