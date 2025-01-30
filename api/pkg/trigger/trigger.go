package trigger

import (
	"time"
)

type EventSourceType string
type TimeInterval string

func (t *TimeInterval) String() {
	panic("unimplemented")
}

const (
	EventSourceTypeTimeDriven EventSourceType = "TIME_DRIVEN"
	EventSourceTypeAPIDriven  EventSourceType = "API_DRIVEN"

	TimeIntervalEveryDay   TimeInterval = "EVERY_DAY"
	TimeIntervalEveryHour  TimeInterval = "EVERY_HOUR"
	TimeIntervalEveryMonth TimeInterval = "EVERY_MONTH"
	TimeIntervalEveryWeek  TimeInterval = "EVERY_WEEK"
)

type Trigger struct {
	id            ID
	createdAt     time.Time
	updatedAt     time.Time
	lastTriggered *time.Time
	workspaceId   WorkspaceID
	deploymentId  DeploymentID
	description   string
	eventSource   EventSourceType
	authToken     *string
	timeInterval  *TimeInterval
}

func (t *Trigger) ID() ID {
	return t.id
}

func (t *Trigger) CreatedAt() time.Time {
	return t.createdAt
}

func (t *Trigger) UpdatedAt() time.Time {
	return t.updatedAt
}

func (t *Trigger) LastTriggered() *time.Time {
	return t.lastTriggered
}

func (t *Trigger) Workspace() WorkspaceID {
	return t.workspaceId
}

func (t *Trigger) Description() string {
	return t.description
}

func (t *Trigger) Deployment() DeploymentID {
	return t.deploymentId
}

func (t *Trigger) EventSource() EventSourceType {
	return t.eventSource
}

func (t *Trigger) AuthToken() *string {
	return t.authToken
}

func (t *Trigger) TimeInterval() *TimeInterval {
	return t.timeInterval
}

func (t *Trigger) SetLastTriggered(lastTriggered time.Time) {
	t.lastTriggered = &lastTriggered
	t.updatedAt = time.Now()
}

func (t *Trigger) SetAuthToken(token string) {
	t.authToken = &token
	t.updatedAt = time.Now()
}

func (t *Trigger) SetEventSource(eventSource EventSourceType) {
	t.eventSource = eventSource
	t.updatedAt = time.Now()
}

func (t *Trigger) SetDescription(description string) {
	t.description = description
	t.updatedAt = time.Now()
}

func (t *Trigger) SetDeployment(deploymentId DeploymentID) {
	t.deploymentId = deploymentId
	t.updatedAt = time.Now()
}

func (t *Trigger) SetTimeInterval(interval TimeInterval) {
	t.timeInterval = &interval
	t.updatedAt = time.Now()
}

func (t *Trigger) SetUpdatedAt(updatedAt time.Time) {
	t.updatedAt = updatedAt
}
