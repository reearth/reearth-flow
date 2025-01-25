package config

type RedisConfig struct {
	Addr     string `default:"localhost:6379"`
	Password string `default:""`
	DB       int    `default:"0"`
}
