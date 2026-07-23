package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/subscriber/pkg/diagnostic"
	"go.mongodb.org/mongo-driver/bson/primitive"
)

type diagnosticSourceSpanDocument struct {
	Length *uint `bson:"length,omitempty"`
	Offset uint  `bson:"offset"`
}

type diagnosticAggregateInfoDocument struct {
	SampleFeatureIDs []string `bson:"sampleFeatureIds"`
	Count            uint64   `bson:"count"`
}

type DiagnosticDocument struct {
	Timestamp            time.Time                        `bson:"timestamp"`
	Aggregated           *diagnosticAggregateInfoDocument `bson:"aggregated,omitempty"`
	SourceSpan           *diagnosticSourceSpanDocument    `bson:"sourceSpan,omitempty"`
	EffectiveDisposition *string                          `bson:"effectiveDisposition,omitempty"`
	NodeID               *string                          `bson:"nodeId,omitempty"`
	ActionType           *string                          `bson:"actionType,omitempty"`
	FeatureID            *string                          `bson:"featureId,omitempty"`
	Help                 *string                          `bson:"help,omitempty"`
	ID                   string                           `bson:"id"`
	JobID                string                           `bson:"jobId"`
	WorkflowID           string                           `bson:"workflowId"`
	Schema               string                           `bson:"schema"`
	Code                 string                           `bson:"code"`
	Category             string                           `bson:"category"`
	Severity             string                           `bson:"severity"`
	Message              string                           `bson:"message"`
}

const JobDiagnosticNodeSegment = "_job"

func normalizedNodeSegment(nodeID *string) string {
	if nodeID != nil && *nodeID != "" {
		return *nodeID
	}
	return JobDiagnosticNodeSegment
}

func NewDiagnosticDocument(event *diagnostic.DiagnosticEvent) DiagnosticDocument {
	nodeSegment := normalizedNodeSegment(event.NodeID)

	doc := DiagnosticDocument{
		Timestamp:            event.Timestamp,
		EffectiveDisposition: event.EffectiveDisposition,
		NodeID:               &nodeSegment,
		ActionType:           event.ActionType,
		FeatureID:            event.FeatureID,
		Help:                 event.Help,
		ID:                   event.JobID + ":" + nodeSegment + ":" + primitive.NewObjectID().Hex(),
		JobID:                event.JobID,
		WorkflowID:           event.WorkflowID,
		Schema:               event.Schema,
		Code:                 event.Code,
		Category:             event.Category,
		Severity:             event.Severity,
		Message:              event.Message,
	}

	if event.SourceSpan != nil {
		doc.SourceSpan = &diagnosticSourceSpanDocument{
			Offset: event.SourceSpan.Offset,
			Length: event.SourceSpan.Length,
		}
	}

	if event.Aggregated != nil {
		doc.Aggregated = &diagnosticAggregateInfoDocument{
			Count:            event.Aggregated.Count,
			SampleFeatureIDs: event.Aggregated.SampleFeatureIds,
		}
	}

	return doc
}
