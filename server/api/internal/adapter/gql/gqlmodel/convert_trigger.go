package gqlmodel

import (
	"fmt"

	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearth-flow/api/pkg/variable"
)

func ToTrigger(t *trigger.Trigger) *Trigger {
	if t == nil {
		return nil
	}

	var timeInterval *TimeInterval
	if t.TimeInterval() != nil {
		ti := ToTimeInterval(*t.TimeInterval())
		timeInterval = ti
	}

	return &Trigger{
		ID:            IDFrom(t.ID()),
		CreatedAt:     t.CreatedAt(),
		UpdatedAt:     t.UpdatedAt(),
		LastTriggered: t.LastTriggered(),
		WorkspaceID:   IDFrom(t.Workspace()),
		DeploymentID:  IDFrom(t.Deployment()),
		Description:   t.Description(),
		EventSource:   ToEventSourceType(t.EventSource()),
		AuthTokenHint: t.AuthTokenHint(),
		TimeInterval:  timeInterval,
		Enabled:       t.Enabled(),
		Variables:     ToVariables(t.Variables()),
		LastRefusal:   ToTriggerRefusal(t.LastRefusal()),
	}
}

// ToTriggerRefusal is nil for no refusal, and for a refusal whose code the
// schema does not name, such as a trigger that stopped being time-driven.
func ToTriggerRefusal(r *trigger.Refusal) *TriggerRefusal {
	if r == nil {
		return nil
	}
	var code TriggerRefusalCode
	switch r.Code {
	case trigger.RefusalNotFound:
		// Only a trigger that exists records a refusal, so what was not found
		// is its deployment.
		code = TriggerRefusalCodeDeploymentNotFound
	case trigger.RefusalTriggerDisabled:
		code = TriggerRefusalCodeTriggerDisabled
	case trigger.RefusalInvalidVariables:
		code = TriggerRefusalCodeInvalidVariables
	case trigger.RefusalNotAllowed:
		code = TriggerRefusalCodeNotAllowed
	case trigger.RefusalConcurrentJobLimit:
		code = TriggerRefusalCodeConcurrentJobLimit
	case trigger.RefusalServerError:
		code = TriggerRefusalCodeServerError
	default:
		return nil
	}
	return &TriggerRefusal{RefusedAt: r.At, Code: code, RetryUntil: r.RetryUntil}
}

// ToTriggerPayload carries authToken only when it was just issued.
func ToTriggerPayload(t *trigger.Trigger, authToken string) *TriggerPayload {
	p := &TriggerPayload{Trigger: ToTrigger(t)}
	if authToken != "" {
		p.AuthToken = &authToken
	}
	return p
}

func ToEventSourceType(t trigger.EventSourceType) EventSourceType {
	switch t {
	case trigger.EventSourceTypeTimeDriven:
		return EventSourceTypeTimeDriven
	case trigger.EventSourceTypeAPIDriven:
		return EventSourceTypeAPIDriven
	default:
		return ""
	}
}

func FromEventSourceType(t EventSourceType) trigger.EventSourceType {
	switch t {
	case EventSourceTypeTimeDriven:
		return trigger.EventSourceTypeTimeDriven
	case EventSourceTypeAPIDriven:
		return trigger.EventSourceTypeAPIDriven
	default:
		return ""
	}
}

func ToTimeInterval(t trigger.TimeInterval) *TimeInterval {
	if t == "" {
		return nil
	}

	var interval TimeInterval
	switch t {
	case trigger.TimeIntervalEveryDay:
		interval = TimeIntervalEveryDay
	case trigger.TimeIntervalEveryHour:
		interval = TimeIntervalEveryHour
	case trigger.TimeIntervalEveryMonth:
		interval = TimeIntervalEveryMonth
	case trigger.TimeIntervalEveryWeek:
		interval = TimeIntervalEveryWeek
	}
	return &interval
}

func FromTimeInterval(t TimeInterval) trigger.TimeInterval {
	switch t {
	case TimeIntervalEveryDay:
		return trigger.TimeIntervalEveryDay
	case TimeIntervalEveryHour:
		return trigger.TimeIntervalEveryHour
	case TimeIntervalEveryMonth:
		return trigger.TimeIntervalEveryMonth
	case TimeIntervalEveryWeek:
		return trigger.TimeIntervalEveryWeek
	default:
		return ""
	}
}

func ToVariables(vars []variable.Variable) []*Variable {
	if len(vars) == 0 {
		return []*Variable{}
	}
	out := make([]*Variable, 0, len(vars))
	for _, v := range vars {
		vt := ParameterType(v.Type)
		out = append(out, &Variable{
			Key:   v.Key,
			Type:  vt,
			Value: v.Value,
		})
	}
	return out
}

func FromVariables(in []*VariableInput) ([]variable.Variable, error) {
	if in == nil {
		return nil, nil
	}
	out := make([]variable.Variable, 0, len(in))
	for _, v := range in {
		if v == nil {
			continue
		}
		t := parameter.Type(v.Type)
		if t == "" {
			return nil, fmt.Errorf("invalid variable type for key %s", v.Key)
		}
		out = append(out, variable.Variable{
			Key:   v.Key,
			Type:  t,
			Value: v.Value,
		})
	}
	return out, nil
}
