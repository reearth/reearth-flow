package cms

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/usecase/gateway"
	"github.com/reearth/reearth-flow/api/pkg/circuitbreaker"
	"github.com/reearth/reearth-flow/api/pkg/cms"
	"github.com/sony/gobreaker/v2"
)

// circuitBreakerCMS is a gateway.CMS decorator that trips a circuit breaker
// when the underlying CMS starts failing consistently. It is intentionally
// stateless apart from the breaker itself: no caching, no retries, no queueing.
//
// Once the breaker is open, calls return circuitbreaker.ErrOpen immediately
// instead of hammering an unhealthy CMS backend.
type circuitBreakerCMS struct {
	next    gateway.CMS
	project *gobreaker.CircuitBreaker[*cms.Project]
	projLst *gobreaker.CircuitBreaker[*cms.ListProjectsOutput]
	asset   *gobreaker.CircuitBreaker[*cms.Asset]
	assLst  *gobreaker.CircuitBreaker[*cms.ListAssetsOutput]
	model   *gobreaker.CircuitBreaker[*cms.Model]
	modLst  *gobreaker.CircuitBreaker[*cms.ListModelsOutput]
	itmLst  *gobreaker.CircuitBreaker[*cms.ListItemsOutput]
	export  *gobreaker.CircuitBreaker[*cms.ExportOutput]
}

// NewCircuitBreakerCMS wraps next with a circuit breaker per method group.
// If next is nil, this returns nil so callers can chain it after an optional
// initialization step without extra nil checks.
//
// Method groups share a breaker when they return the same type; that keeps
// the number of breakers tractable while still isolating CMS from, say, the
// permission or auth backends (which have their own breakers).
func NewCircuitBreakerCMS(next gateway.CMS, cfg circuitbreaker.Config) gateway.CMS {
	if next == nil {
		return nil
	}
	if cfg.Name == "" {
		cfg.Name = "cms"
	}
	return &circuitBreakerCMS{
		next:    next,
		project: circuitbreaker.New[*cms.Project](cfg),
		projLst: circuitbreaker.New[*cms.ListProjectsOutput](cfg),
		asset:   circuitbreaker.New[*cms.Asset](cfg),
		assLst:  circuitbreaker.New[*cms.ListAssetsOutput](cfg),
		model:   circuitbreaker.New[*cms.Model](cfg),
		modLst:  circuitbreaker.New[*cms.ListModelsOutput](cfg),
		itmLst:  circuitbreaker.New[*cms.ListItemsOutput](cfg),
		export:  circuitbreaker.New[*cms.ExportOutput](cfg),
	}
}

func (c *circuitBreakerCMS) GetProject(ctx context.Context, projectIDOrAlias string) (*cms.Project, error) {
	return c.project.Execute(func() (*cms.Project, error) {
		return c.next.GetProject(ctx, projectIDOrAlias)
	})
}

func (c *circuitBreakerCMS) ListProjects(ctx context.Context, input cms.ListProjectsInput) (*cms.ListProjectsOutput, error) {
	return c.projLst.Execute(func() (*cms.ListProjectsOutput, error) {
		return c.next.ListProjects(ctx, input)
	})
}

func (c *circuitBreakerCMS) GetAsset(ctx context.Context, input cms.GetAssetInput) (*cms.Asset, error) {
	return c.asset.Execute(func() (*cms.Asset, error) {
		return c.next.GetAsset(ctx, input)
	})
}

func (c *circuitBreakerCMS) ListAssets(ctx context.Context, input cms.ListAssetsInput) (*cms.ListAssetsOutput, error) {
	return c.assLst.Execute(func() (*cms.ListAssetsOutput, error) {
		return c.next.ListAssets(ctx, input)
	})
}

func (c *circuitBreakerCMS) GetModel(ctx context.Context, input cms.GetModelInput) (*cms.Model, error) {
	return c.model.Execute(func() (*cms.Model, error) {
		return c.next.GetModel(ctx, input)
	})
}

func (c *circuitBreakerCMS) ListModels(ctx context.Context, input cms.ListModelsInput) (*cms.ListModelsOutput, error) {
	return c.modLst.Execute(func() (*cms.ListModelsOutput, error) {
		return c.next.ListModels(ctx, input)
	})
}

func (c *circuitBreakerCMS) ListItems(ctx context.Context, input cms.ListItemsInput) (*cms.ListItemsOutput, error) {
	return c.itmLst.Execute(func() (*cms.ListItemsOutput, error) {
		return c.next.ListItems(ctx, input)
	})
}

func (c *circuitBreakerCMS) GetModelExportURL(ctx context.Context, input cms.ModelExportInput) (*cms.ExportOutput, error) {
	return c.export.Execute(func() (*cms.ExportOutput, error) {
		return c.next.GetModelExportURL(ctx, input)
	})
}

func (c *circuitBreakerCMS) GetModelGeoJSONExportURL(ctx context.Context, input cms.ExportInput) (*cms.ExportOutput, error) {
	return c.export.Execute(func() (*cms.ExportOutput, error) {
		return c.next.GetModelGeoJSONExportURL(ctx, input)
	})
}
