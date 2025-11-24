package trigger

import (
	"time"
)

type (
	EventSourceType string
	TimeInterval    string
)

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
	createdAt     time.Time
	updatedAt     time.Time
	lastTriggered *time.Time
	authToken     *string
	timeInterval  *TimeInterval
	variables     map[string]string
	description   string
	eventSource   EventSourceType
	id            ID
	workspaceId   WorkspaceID
	deploymentId  DeploymentID
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

func (t *Trigger) Variables() map[string]string {
	return t.variables
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

func (t *Trigger) SetVariables(variables map[string]string) {
	t.variables = variables
	t.updatedAt = time.Now()
}

func (t *Trigger) SetUpdatedAt(updatedAt time.Time) {
	t.updatedAt = updatedAt
}
