package gqlmodel

import (
	"github.com/reearth/reearth-flow/api/pkg/trigger"
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
		AuthToken:     t.AuthToken(),
		TimeInterval:  timeInterval,
	}
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
