package asyncq

import (
	"time"

	"github.com/hibiken/asynq"
	"github.com/redis/go-redis/v9"
)

type Config struct {
	RedisAddr      string                                      `json:"redis_addr"`
	RedisPassword  string                                      `json:"redis_password"`
	RedisDB        int                                         `json:"redis_db"`
	Concurrency    int                                         `json:"concurrency"`
	MaxRetry       int                                         `json:"max_retry"`
	RetryDelayFunc func(int, error, *asynq.Task) time.Duration `json:"-"`
	Queues         map[string]int                              `json:"queues"`
}

func DefaultConfig() *Config {
	return &Config{
		RedisAddr:      "localhost:6379",
		RedisPassword:  "",
		RedisDB:        0,
		Concurrency:    10,
		MaxRetry:       3,
		RetryDelayFunc: asynq.DefaultRetryDelayFunc,
		Queues: map[string]int{
			"critical": 6,
			"default":  3,
			"low":      1,
		},
	}
}

func (c *Config) GetRedisClientOpt() asynq.RedisClientOpt {
	return asynq.RedisClientOpt{
		Addr:     c.RedisAddr,
		Password: c.RedisPassword,
		DB:       c.RedisDB,
	}
}

func (c *Config) GetRedisConnOpt() *redis.Options {
	return &redis.Options{
		Addr:     c.RedisAddr,
		Password: c.RedisPassword,
		DB:       c.RedisDB,
	}
}
