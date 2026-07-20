package diagnostic

import (
	"errors"
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

// ErrInvalidDiagnostic is returned by Builder.Build when a required field
// (Code) was not set.
var ErrInvalidDiagnostic = errors.New("diagnostic: code is required")

type Builder struct {
	d *Diagnostic
}

func NewBuilder() *Builder {
	return &Builder{d: &Diagnostic{}}
}

func (b *Builder) Build() (*Diagnostic, error) {
	if b.d.code == "" {
		return nil, ErrInvalidDiagnostic
	}
	return b.d, nil
}

func (b *Builder) MustBuild() *Diagnostic {
	r, err := b.Build()
	if err != nil {
		panic(err)
	}
	return r
}

func (b *Builder) JobID(jobID id.JobID) *Builder {
	b.d.jobID = jobID
	return b
}

func (b *Builder) NodeID(nodeID *string) *Builder {
	b.d.nodeID = nodeID
	return b
}

func (b *Builder) Timestamp(timestamp time.Time) *Builder {
	b.d.timestamp = timestamp
	return b
}

func (b *Builder) Code(code string) *Builder {
	b.d.code = code
	return b
}

func (b *Builder) Category(category string) *Builder {
	b.d.category = category
	return b
}

func (b *Builder) Severity(severity string) *Builder {
	b.d.severity = severity
	return b
}

func (b *Builder) EffectiveDisposition(effectiveDisposition *string) *Builder {
	b.d.effectiveDisposition = effectiveDisposition
	return b
}

func (b *Builder) ActionType(actionType *string) *Builder {
	b.d.actionType = actionType
	return b
}

func (b *Builder) FeatureID(featureID *string) *Builder {
	b.d.featureID = featureID
	return b
}

func (b *Builder) Message(message string) *Builder {
	b.d.message = message
	return b
}

func (b *Builder) Help(help *string) *Builder {
	b.d.help = help
	return b
}

func (b *Builder) Aggregated(aggregated *AggregateInfo) *Builder {
	b.d.aggregated = aggregated
	return b
}

func (b *Builder) SourceSpan(sourceSpan *SourceSpan) *Builder {
	b.d.sourceSpan = sourceSpan
	return b
}

// Terminal marks this row as persisted at job-completion merge time (see
// Diagnostic.terminal). Defaults to false; only mongodoc's Model() sets it
// true, for rows whose stored schema is the job-complete.v1 tag.
func (b *Builder) Terminal(terminal bool) *Builder {
	b.d.terminal = terminal
	return b
}
