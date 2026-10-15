package internalapi

import (
	"context"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	flowv1 "github.com/reearth/reearth-proto/gen/go/flow/v1"
	"google.golang.org/protobuf/types/known/timestamppb"
)

func toProject(ctx context.Context, p *project.Project) *flowv1.Project {
	if p == nil {
		return nil
	}
	starred := false
	if u := adapter.User(ctx); u != nil {
		starred = p.IsStarredBy(accountsid.UserID(u.ID()))
	}
	return &flowv1.Project{
		Id:          p.ID().String(),
		WorkspaceId: p.Workspace().String(),
		Name:        p.Name(),
		Alias:       p.Alias(),
		Description: p.Description(),
		Visibility:  toVisibility(p.Visibility()),
		Readme:      p.Readme(),
		License:     p.License(),
		Topics:      p.Topics(),
		StarCount:   p.StarCount(),
		Starred:     starred,
		IsArchived:  p.IsArchived(),
		CreatedAt:   timestamppb.New(p.CreatedAt()),
		UpdatedAt:   timestamppb.New(p.UpdatedAt()),
	}
}

func toProjects(ctx context.Context, ps []*project.Project) []*flowv1.Project {
	res := make([]*flowv1.Project, 0, len(ps))
	for _, p := range ps {
		if p != nil {
			res = append(res, toProject(ctx, p))
		}
	}
	return res
}

func toVisibility(v project.Visibility) flowv1.ProjectVisibility {
	switch v {
	case project.VisibilityPublic:
		return flowv1.ProjectVisibility_PROJECT_VISIBILITY_PUBLIC
	case project.VisibilityPrivate:
		return flowv1.ProjectVisibility_PROJECT_VISIBILITY_PRIVATE
	}
	return flowv1.ProjectVisibility_PROJECT_VISIBILITY_UNSPECIFIED
}

// fromVisibility returns nil for UNSPECIFIED, and ok false for a value it does not know.
func fromVisibility(v flowv1.ProjectVisibility) (_ *project.Visibility, ok bool) {
	var res project.Visibility
	switch v {
	case flowv1.ProjectVisibility_PROJECT_VISIBILITY_UNSPECIFIED:
		return nil, true
	case flowv1.ProjectVisibility_PROJECT_VISIBILITY_PUBLIC:
		res = project.VisibilityPublic
	case flowv1.ProjectVisibility_PROJECT_VISIBILITY_PRIVATE:
		res = project.VisibilityPrivate
	default:
		return nil, false
	}
	return &res, true
}

func toDeployment(d *deployment.Deployment) *flowv1.Deployment {
	if d == nil {
		return nil
	}
	var pid *string
	if d.Project() != nil {
		s := d.Project().String()
		pid = &s
	}
	return &flowv1.Deployment{
		Id:          d.ID().String(),
		WorkspaceId: d.Workspace().String(),
		ProjectId:   pid,
		Version:     d.Version(),
		Description: d.Description(),
		IsHead:      d.IsHead(),
		CreatedAt:   timestamppb.New(d.CreatedAt()),
		UpdatedAt:   timestamppb.New(d.UpdatedAt()),
	}
}

func toDeployments(ds []*deployment.Deployment) []*flowv1.Deployment {
	res := make([]*flowv1.Deployment, 0, len(ds))
	for _, d := range ds {
		if d != nil {
			res = append(res, toDeployment(d))
		}
	}
	return res
}

func toTrigger(t *trigger.Trigger) *flowv1.Trigger {
	if t == nil {
		return nil
	}
	res := &flowv1.Trigger{
		Id:              t.ID().String(),
		WorkspaceId:     t.Workspace().String(),
		DeploymentId:    t.Deployment().String(),
		EventSource:     toEventSource(t.EventSource()),
		Enabled:         t.Enabled(),
		Description:     t.Description(),
		LastTriggeredAt: timestampOrNil(t.LastTriggered()),
		CreatedAt:       timestamppb.New(t.CreatedAt()),
		UpdatedAt:       timestamppb.New(t.UpdatedAt()),
	}
	if t.TimeInterval() != nil {
		res.TimeInterval = toTimeInterval(*t.TimeInterval())
	}
	return res
}

func toTriggers(ts []*trigger.Trigger) []*flowv1.Trigger {
	res := make([]*flowv1.Trigger, 0, len(ts))
	for _, t := range ts {
		if t != nil {
			res = append(res, toTrigger(t))
		}
	}
	return res
}

func toEventSource(e trigger.EventSourceType) flowv1.TriggerEventSource {
	switch e {
	case trigger.EventSourceTypeTimeDriven:
		return flowv1.TriggerEventSource_TRIGGER_EVENT_SOURCE_TIME_DRIVEN
	case trigger.EventSourceTypeAPIDriven:
		return flowv1.TriggerEventSource_TRIGGER_EVENT_SOURCE_API_DRIVEN
	}
	return flowv1.TriggerEventSource_TRIGGER_EVENT_SOURCE_UNSPECIFIED
}

func toTimeInterval(i trigger.TimeInterval) flowv1.TriggerTimeInterval {
	switch i {
	case trigger.TimeIntervalEveryHour:
		return flowv1.TriggerTimeInterval_TRIGGER_TIME_INTERVAL_EVERY_HOUR
	case trigger.TimeIntervalEveryDay:
		return flowv1.TriggerTimeInterval_TRIGGER_TIME_INTERVAL_EVERY_DAY
	case trigger.TimeIntervalEveryWeek:
		return flowv1.TriggerTimeInterval_TRIGGER_TIME_INTERVAL_EVERY_WEEK
	case trigger.TimeIntervalEveryMonth:
		return flowv1.TriggerTimeInterval_TRIGGER_TIME_INTERVAL_EVERY_MONTH
	}
	return flowv1.TriggerTimeInterval_TRIGGER_TIME_INTERVAL_UNSPECIFIED
}

func toJob(j *job.Job) *flowv1.Job {
	if j == nil {
		return nil
	}
	res := &flowv1.Job{
		Id:          j.ID().String(),
		WorkspaceId: j.Workspace().String(),
		Status:      toJobStatus(j.Status()),
		StartedAt:   timestamppb.New(j.StartedAt()),
		CompletedAt: timestampOrNil(j.CompletedAt()),
	}
	if j.Deployment() != nil {
		s := j.Deployment().String()
		res.DeploymentId = &s
	}
	if j.ProjectID() != nil {
		s := j.ProjectID().String()
		res.ProjectId = &s
	}
	return res
}

func toJobs(js []*job.Job) []*flowv1.Job {
	res := make([]*flowv1.Job, 0, len(js))
	for _, j := range js {
		if j != nil {
			res = append(res, toJob(j))
		}
	}
	return res
}

func toJobStatus(s job.Status) flowv1.JobStatus {
	switch s {
	case job.StatusPending:
		return flowv1.JobStatus_JOB_STATUS_PENDING
	case job.StatusRunning:
		return flowv1.JobStatus_JOB_STATUS_RUNNING
	case job.StatusCompleted:
		return flowv1.JobStatus_JOB_STATUS_COMPLETED
	case job.StatusFailed:
		return flowv1.JobStatus_JOB_STATUS_FAILED
	case job.StatusCancelled:
		return flowv1.JobStatus_JOB_STATUS_CANCELLED
	}
	return flowv1.JobStatus_JOB_STATUS_UNSPECIFIED
}

func toPageInfo(info *interfaces.PageBasedInfo, page, size int) *flowv1.PageInfo {
	res := &flowv1.PageInfo{Page: int32(page), PageSize: int32(size)}
	if info != nil && info.PageInfo != nil {
		res.TotalCount = info.TotalCount
	}
	return res
}

func timestampOrNil(t *time.Time) *timestamppb.Timestamp {
	if t == nil {
		return nil
	}
	return timestamppb.New(*t)
}
