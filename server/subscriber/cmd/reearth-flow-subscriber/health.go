package main

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"time"

	"cloud.google.com/go/pubsub/v2"
	"github.com/hellofresh/health-go/v5"
	healthMongo "github.com/hellofresh/health-go/v5/checks/mongo"
	healthRedis "github.com/hellofresh/health-go/v5/checks/redis"
	"github.com/redis/go-redis/v9"
	"go.mongodb.org/mongo-driver/mongo"
)

type HealthChecker struct {
	health *health.Health
	config *Config
}

func NewHealthChecker(
	conf *Config,
	ver string,
	redisClient *redis.Client,
	mongoClient *mongo.Client,
	pubsubClient *pubsub.Client,
) *HealthChecker {
	checks := []health.Config{}

	if redisClient != nil && conf.RedisURL != "" {
		checks = append(checks, health.Config{
			Name:      "redis",
			Timeout:   time.Second * 5,
			SkipOnErr: false,
			Check:     healthRedis.New(healthRedis.Config{DSN: conf.RedisURL}),
		})
	}

	if mongoClient != nil && conf.DB != "" {
		checks = append(checks, health.Config{
			Name:      "db",
			Timeout:   time.Second * 5,
			SkipOnErr: false,
			Check:     healthMongo.New(healthMongo.Config{DSN: conf.DB}),
		})
	}

	if pubsubClient != nil && conf.GCPProject != "" {
		checks = append(checks, health.Config{
			Name:      "pubsub",
			Timeout:   time.Second * 10,
			SkipOnErr: false,
			Check:     createPubSubCheck(pubsubClient, conf),
		})
	}

	h, err := health.New(health.WithComponent(health.Component{
		Name:    "reearth-flow-subscriber",
		Version: ver,
	}), health.WithChecks(checks...))
	if err != nil {
		h, _ = health.New(health.WithComponent(health.Component{
			Name:    "reearth-flow-subscriber",
			Version: ver,
		}))
	}

	return &HealthChecker{
		health: h,
		config: conf,
	}
}

func (hc *HealthChecker) Handler() http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		if hc.config.HealthCheckUsername != "" && hc.config.HealthCheckPassword != "" {
			username, password, ok := r.BasicAuth()
			if !ok || username != hc.config.HealthCheckUsername || password != hc.config.HealthCheckPassword {
				w.Header().Set("Content-Type", "application/json")
				w.WriteHeader(http.StatusUnauthorized)
				_ = json.NewEncoder(w).Encode(map[string]string{
					"error": "unauthorized",
				})
				return
			}
		}

		hc.health.Handler().ServeHTTP(w, r)
	}
}

func (hc *HealthChecker) Check(ctx context.Context) error {
	result := hc.health.Measure(ctx)
	if len(result.Failures) > 0 {
		return &HealthCheckError{Failures: result.Failures}
	}
	return nil
}

type HealthCheckError struct {
	Failures map[string]string
}

func (e *HealthCheckError) Error() string {
	return "health check failed"
}

func createPubSubCheck(client *pubsub.Client, conf *Config) func(ctx context.Context) error {
	return func(ctx context.Context) error {
		subscriptions := []string{
			conf.LogSubscriptionID,
			conf.NodeSubscriptionID,
			conf.UserFacingLogSubscriptionID,
			conf.JobCompleteSubscriptionID,
		}

		for _, subID := range subscriptions {
			if subID != "" {
				sub := client.Subscriber(subID)
				if sub.ID() == "" {
					return fmt.Errorf("invalid subscription: %s", subID)
				}
				return nil
			}
		}

		return fmt.Errorf("no subscriptions configured")
	}
}
