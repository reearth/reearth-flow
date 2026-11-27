package postgres

import (
	"context"
	"errors"
	"time"

	"github.com/jackc/pgx/v5"
	"github.com/reearth/reearth-flow/api/internal/infrastructure/postgres/gen"
	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/diagnostic"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearthx/pgxx"
	"github.com/reearth/reearthx/rerror"
)

// Marks rows written by the job-complete merge; the subscriber's streamed rows carry the event schema.
const terminalDiagnosticSchema = "job-complete.v1"

// Keeps terminal row IDs identical to the Mongo repo's.
const jobDiagnosticNodeSegment = "_job"

type NodeDiagnostics struct {
	c *pgxx.Client
}

var _ repo.NodeDiagnostics = (*NodeDiagnostics)(nil)

func NewNodeDiagnostics(c *pgxx.Client) *NodeDiagnostics {
	return &NodeDiagnostics{c: c}
}

func (r *NodeDiagnostics) q(ctx context.Context) *gen.Queries {
	return gen.New(r.c.DB(ctx))
}

// Empty nodeID reads the job-level bucket.
func (r *NodeDiagnostics) FindByJobNodeID(ctx context.Context, jobID id.JobID, nodeID string) ([]*diagnostic.Diagnostic, error) {
	var (
		rows []gen.NodeDiagnostic
		err  error
	)
	if nodeID == "" {
		rows, err = r.q(ctx).ListJobLevelNodeDiagnostics(ctx, jobID.String())
	} else {
		rows, err = r.q(ctx).ListNodeDiagnosticsByJobNodeID(ctx, gen.ListNodeDiagnosticsByJobNodeIDParams{
			JobID:  jobID.String(),
			NodeID: &nodeID,
		})
	}
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nodeDiagnosticsFromRows(ctx, rows)
}

func (r *NodeDiagnostics) FindByJobID(ctx context.Context, jobID id.JobID) ([]*diagnostic.Diagnostic, error) {
	rows, err := r.q(ctx).ListNodeDiagnosticsByJobID(ctx, jobID.String())
	if err != nil {
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nodeDiagnosticsFromRows(ctx, rows)
}

func (r *NodeDiagnostics) FindJobSummary(ctx context.Context, jobID id.JobID) (*uint64, error) {
	row, err := r.q(ctx).GetJobDiagnosticSummary(ctx, jobID.String())
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return nil, nil
		}
		return nil, rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	count := uint64(row.DroppedEventCount) //nolint:gosec
	return &count, nil
}

func (r *NodeDiagnostics) SaveTerminalDiagnostics(
	ctx context.Context,
	jobID id.JobID,
	workflowID string,
	timestamp time.Time,
	failedNodes []*diagnostic.Diagnostic,
	aggregated []*diagnostic.Diagnostic,
	droppedEventCount *uint64,
) error {
	err := r.c.WithinTransaction(ctx, func(ctx context.Context) error {
		q := r.q(ctx)
		for _, d := range failedNodes {
			if err := q.UpsertNodeDiagnostic(ctx, terminalDiagnosticParams(jobID, workflowID, d, "failed")); err != nil {
				return err
			}
		}
		// One row each, not nested, so every diagnostic stays visible to FindByJobNodeID.
		for _, d := range aggregated {
			if err := q.UpsertNodeDiagnostic(ctx, terminalDiagnosticParams(jobID, workflowID, d, "aggregated")); err != nil {
				return err
			}
		}
		if droppedEventCount == nil {
			return nil
		}
		return q.UpsertJobDiagnosticSummary(ctx, gen.UpsertJobDiagnosticSummaryParams{
			JobID:             jobID.String(),
			Ts:                timestamp,
			DroppedEventCount: int64(*droppedEventCount), //nolint:gosec
		})
	})
	if err != nil {
		return rerror.ErrInternalByWithContext(ctx, pgxx.WrapError(err))
	}
	return nil
}

// The ID is deterministic so redeliveries upsert instead of duplicating.
func terminalDiagnosticParams(jobID id.JobID, workflowID string, d *diagnostic.Diagnostic, kind string) gen.UpsertNodeDiagnosticParams {
	var nodeID *string
	nodeSegment := jobDiagnosticNodeSegment
	if n := d.NodeID(); n != nil && *n != "" {
		nodeID = n
		nodeSegment = *n
	}

	p := gen.UpsertNodeDiagnosticParams{
		ID:                   jobID.String() + ":" + nodeSegment + ":" + kind + ":" + d.Code(),
		JobID:                jobID.String(),
		WorkflowID:           workflowID,
		NodeID:               nodeID,
		Ts:                   d.Timestamp(),
		Schema:               terminalDiagnosticSchema,
		Code:                 d.Code(),
		Category:             d.Category(),
		Severity:             d.Severity(),
		Message:              []byte(d.Message()),
		EffectiveDisposition: d.EffectiveDisposition(),
		ActionType:           d.ActionType(),
		FeatureID:            d.FeatureID(),
		Help:                 d.Help(),
	}
	if ss := d.SourceSpan(); ss != nil {
		offset := int64(ss.Offset()) //nolint:gosec
		p.SourceSpanOffset = &offset
		if l := ss.Length(); l != nil {
			length := int64(*l) //nolint:gosec
			p.SourceSpanLength = &length
		}
	}
	if agg := d.Aggregated(); agg != nil {
		count := int64(agg.Count()) //nolint:gosec
		p.AggregatedCount = &count
		p.AggregatedSampleFeatureIds = agg.SampleFeatureIDs()
	}
	return p
}

func nodeDiagnosticsFromRows(ctx context.Context, rows []gen.NodeDiagnostic) ([]*diagnostic.Diagnostic, error) {
	out := make([]*diagnostic.Diagnostic, 0, len(rows))
	for _, row := range rows {
		d, err := nodeDiagnosticFromRow(row)
		if err != nil {
			return nil, rerror.ErrInternalByWithContext(ctx, err)
		}
		out = append(out, d)
	}
	return out, nil
}

func nodeDiagnosticFromRow(row gen.NodeDiagnostic) (*diagnostic.Diagnostic, error) {
	jobID, err := id.JobIDFrom(row.JobID)
	if err != nil {
		return nil, err
	}

	b := diagnostic.NewBuilder().
		JobID(jobID).
		Timestamp(row.Ts).
		Code(row.Code).
		Category(row.Category).
		Severity(row.Severity).
		EffectiveDisposition(row.EffectiveDisposition).
		NodeID(row.NodeID).
		ActionType(row.ActionType).
		FeatureID(row.FeatureID).
		Message(string(row.Message)).
		Help(row.Help).
		Terminal(row.Schema == terminalDiagnosticSchema)

	if row.AggregatedCount != nil {
		b = b.Aggregated(diagnostic.NewAggregateInfo(uint64(*row.AggregatedCount), row.AggregatedSampleFeatureIds)) //nolint:gosec
	}
	if row.SourceSpanOffset != nil {
		var length *uint
		if row.SourceSpanLength != nil {
			l := uint(*row.SourceSpanLength) //nolint:gosec
			length = &l
		}
		b = b.SourceSpan(diagnostic.NewSourceSpan(uint(*row.SourceSpanOffset), length)) //nolint:gosec
	}

	return b.Build()
}
