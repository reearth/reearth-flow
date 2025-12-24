package redis

import (
	"context"
	"time"

	"github.com/redis/go-redis/v9"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/codes"
	"go.opentelemetry.io/otel/trace"
)

const tracerName = "github.com/reearth/reearth-flow/subscriber/internal/infrastructure/redis"

type RedisClient interface {
	Set(ctx context.Context, key string, value interface{}, expiration time.Duration) *redis.StatusCmd
	LPush(ctx context.Context, key string, values ...interface{}) *redis.IntCmd
	Expire(ctx context.Context, key string, expiration time.Duration) *redis.BoolCmd
}

type RedisStorage struct {
	client RedisClient
	tracer trace.Tracer
}

func NewRedisStorage(client RedisClient) *RedisStorage {
	return &RedisStorage{
		client: client,
		tracer: otel.Tracer(tracerName),
	}
}

func (r *RedisStorage) tracedSet(ctx context.Context, key string, value interface{}, expiration time.Duration) error {
	ctx, span := r.tracer.Start(ctx, "redis.SET",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			attribute.String("db.system", "redis"),
			attribute.String("db.operation", "SET"),
			attribute.String("db.redis.key", key),
		),
	)
	defer span.End()

	if err := r.client.Set(ctx, key, value, expiration).Err(); err != nil {
		span.SetStatus(codes.Error, err.Error())
		span.RecordError(err)
		return err
	}
	span.SetStatus(codes.Ok, "")
	return nil
}

func (r *RedisStorage) tracedLPush(ctx context.Context, key string, values ...interface{}) error {
	ctx, span := r.tracer.Start(ctx, "redis.LPUSH",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			attribute.String("db.system", "redis"),
			attribute.String("db.operation", "LPUSH"),
			attribute.String("db.redis.key", key),
			attribute.Int("db.redis.values_count", len(values)),
		),
	)
	defer span.End()

	if err := r.client.LPush(ctx, key, values...).Err(); err != nil {
		span.SetStatus(codes.Error, err.Error())
		span.RecordError(err)
		return err
	}
	span.SetStatus(codes.Ok, "")
	return nil
}

func (r *RedisStorage) tracedExpire(ctx context.Context, key string, expiration time.Duration) error {
	ctx, span := r.tracer.Start(ctx, "redis.EXPIRE",
		trace.WithSpanKind(trace.SpanKindClient),
		trace.WithAttributes(
			attribute.String("db.system", "redis"),
			attribute.String("db.operation", "EXPIRE"),
			attribute.String("db.redis.key", key),
			attribute.Int64("db.redis.ttl_seconds", int64(expiration.Seconds())),
		),
	)
	defer span.End()

	if err := r.client.Expire(ctx, key, expiration).Err(); err != nil {
		span.SetStatus(codes.Error, err.Error())
		span.RecordError(err)
		return err
	}
	span.SetStatus(codes.Ok, "")
	return nil
}
