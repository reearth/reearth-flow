package mongodoc

import (
	"time"

	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"golang.org/x/exp/slices"
)

type TriggerDocument struct {
	CreatedAt     time.Time         `bson:"createdat"`
	UpdatedAt     time.Time         `bson:"updatedat"`
	LastTriggered time.Time         `bson:"lasttriggered,omitempty"`
	Variables     map[string]string `bson:"variables,omitempty"`
	ID            string            `bson:"id"`
	WorkspaceID   string            `bson:"workspaceid"`
	DeploymentID  string            `bson:"deploymentid"`
	Description   string            `bson:"description"`
	EventSource   string            `bson:"eventsource"`
	TimeInterval  string            `bson:"timeinterval,omitempty"`
	AuthToken     string            `bson:"authtoken,omitempty"`
}

type TriggerConsumer = Consumer[*TriggerDocument, *trigger.Trigger]

func NewTriggerConsumer(workspaces []id.WorkspaceID) *TriggerConsumer {
	return NewConsumer[*TriggerDocument, *trigger.Trigger](func(t *trigger.Trigger) bool {
		return workspaces == nil || slices.Contains(workspaces, t.Workspace())
	})
}

func NewTrigger(t *trigger.Trigger) (*TriggerDocument, string) {
	tid := t.ID().String()

	doc := &TriggerDocument{
		ID:           tid,
		WorkspaceID:  t.Workspace().String(),
		DeploymentID: t.Deployment().String(),
		Description:  t.Description(),
		EventSource:  string(t.EventSource()),
		CreatedAt:    t.CreatedAt(),
		UpdatedAt:    t.UpdatedAt(),
	}

	if timeInterval := t.TimeInterval(); timeInterval != nil {
		ti := string(*timeInterval)
		doc.TimeInterval = ti
	}

	if authToken := t.AuthToken(); authToken != nil {
		at := string(*authToken)
		doc.AuthToken = at
	}

	if lastTriggered := t.LastTriggered(); lastTriggered != nil {
		doc.LastTriggered = *lastTriggered
	}

	if variables := t.Variables(); variables != nil {
		doc.Variables = variables
	}

	return doc, tid
}

func (d *TriggerDocument) Model() (*trigger.Trigger, error) {
	tid, err := id.TriggerIDFrom(d.ID)
	if err != nil {
		return nil, err
	}

	wid, err := id.WorkspaceIDFrom(d.WorkspaceID)
	if err != nil {
		return nil, err
	}

	did, err := id.DeploymentIDFrom(d.DeploymentID)
	if err != nil {
		return nil, err
	}

	eventSource := trigger.EventSourceType(d.EventSource)
	timeInterval := trigger.TimeInterval(d.TimeInterval)

	b := trigger.New().
		ID(tid).
		Workspace(wid).
		Deployment(did).
		Description(d.Description).
		EventSource(eventSource).
		TimeInterval(timeInterval).
		AuthToken(d.AuthToken).
		UpdatedAt(d.UpdatedAt).
		LastTriggered(d.LastTriggered)

	if len(d.Variables) > 0 {
		b = b.Variables(d.Variables)
	}

	return b.Build()
}
