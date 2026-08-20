package postgres

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/gen"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/workerconfig"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
)

type WorkerConfig struct {
	c *pgxx.Client
}

var _ repo.WorkerConfig = (*WorkerConfig)(nil)

func NewWorkerConfig(c *pgxx.Client) *WorkerConfig {
	return &WorkerConfig{c: c}
}

func (r *WorkerConfig) q(ctx context.Context) *gen.Queries {
	return gen.New(r.c.DB(ctx))
}

func (r *WorkerConfig) FindByID(ctx context.Context, wid id.WorkerConfigID) (*workerconfig.WorkerConfig, error) {
	row, err := r.q(ctx).GetWorkerConfig(ctx, wid.String())
	if err != nil {
		return nil, pgxx.MapError(err)
	}
	return workerConfigFromRow(row)
}

func (r *WorkerConfig) FindByIDs(ctx context.Context, ids []id.WorkerConfigID) ([]*workerconfig.WorkerConfig, error) {
	if len(ids) == 0 {
		return nil, nil
	}
	strs := make([]string, len(ids))
	for i, wid := range ids {
		strs[i] = wid.String()
	}
	rows, err := r.q(ctx).ListWorkerConfigsByIDs(ctx, strs)
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	items := make([]*workerconfig.WorkerConfig, 0, len(rows))
	for _, row := range rows {
		cfg, err := workerConfigFromRow(row)
		if err != nil {
			return nil, err
		}
		items = append(items, cfg)
	}
	return pgxx.OrderByIDs(strs, items, func(c *workerconfig.WorkerConfig) string { return c.ID().String() }), nil
}

func (r *WorkerConfig) FindAll(ctx context.Context) (*workerconfig.WorkerConfig, error) {
	rows, err := r.q(ctx).ListWorkerConfigs(ctx)
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	if len(rows) == 0 {
		return nil, nil
	}
	return workerConfigFromRow(rows[0])
}

func (r *WorkerConfig) Save(ctx context.Context, cfg *workerconfig.WorkerConfig) error {
	if err := r.q(ctx).UpsertWorkerConfig(ctx, workerConfigToParams(cfg)); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func (r *WorkerConfig) Remove(ctx context.Context, wid id.WorkerConfigID) error {
	if err := r.q(ctx).DeleteWorkerConfig(ctx, wid.String()); err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

func workerConfigToParams(cfg *workerconfig.WorkerConfig) gen.UpsertWorkerConfigParams {
	return gen.UpsertWorkerConfigParams{
		ID:                    cfg.ID().String(),
		MachineType:           cfg.MachineType(),
		ComputeCpuMilli:       intToInt32Ptr(cfg.ComputeCpuMilli()),
		ComputeMemoryMib:      intToInt32Ptr(cfg.ComputeMemoryMib()),
		BootDiskSizeGb:        intToInt32Ptr(cfg.BootDiskSizeGB()),
		TaskCount:             intToInt32Ptr(cfg.TaskCount()),
		MaxConcurrency:        intToInt32Ptr(cfg.MaxConcurrency()),
		ThreadPoolSize:        intToInt32Ptr(cfg.ThreadPoolSize()),
		ChannelBufferSize:     intToInt32Ptr(cfg.ChannelBufferSize()),
		FeatureFlushThreshold: intToInt32Ptr(cfg.FeatureFlushThreshold()),
		NodeStatusDelayMilli:  intToInt32Ptr(cfg.NodeStatusPropagationDelayMilli()),
		CreatedAt:             cfg.CreatedAt(),
		UpdatedAt:             cfg.UpdatedAt(),
	}
}

func workerConfigFromRow(row gen.WorkerConfig) (*workerconfig.WorkerConfig, error) {
	wid, err := id.WorkerConfigIDFrom(row.ID)
	if err != nil {
		return nil, err
	}
	return workerconfig.NewBuilder().
		ID(wid).
		MachineType(row.MachineType).
		ComputeCpuMilli(int32ToIntPtr(row.ComputeCpuMilli)).
		ComputeMemoryMib(int32ToIntPtr(row.ComputeMemoryMib)).
		BootDiskSizeGB(int32ToIntPtr(row.BootDiskSizeGb)).
		TaskCount(int32ToIntPtr(row.TaskCount)).
		MaxConcurrency(int32ToIntPtr(row.MaxConcurrency)).
		ThreadPoolSize(int32ToIntPtr(row.ThreadPoolSize)).
		ChannelBufferSize(int32ToIntPtr(row.ChannelBufferSize)).
		FeatureFlushThreshold(int32ToIntPtr(row.FeatureFlushThreshold)).
		NodeStatusPropagationDelayMilli(int32ToIntPtr(row.NodeStatusDelayMilli)).
		CreatedAt(row.CreatedAt).
		UpdatedAt(row.UpdatedAt).
		Build()
}

func intToInt32Ptr(v *int) *int32 {
	if v == nil {
		return nil
	}
	v32 := int32(*v)
	return &v32
}

func int32ToIntPtr(v *int32) *int {
	if v == nil {
		return nil
	}
	vi := int(*v)
	return &vi
}
