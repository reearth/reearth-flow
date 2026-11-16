package internalapi

import (
	"context"
	"net/url"
	"path"
	"slices"
	"time"

	accountsid "github.com/reearth/reearth-accounts/server/pkg/id"
	"github.com/reearth/reearth-flow/api/internal/adapter"
	"github.com/reearth/reearth-flow/api/internal/usecase/interfaces"
	"github.com/reearth/reearth-flow/api/pkg/deployment"
	"github.com/reearth/reearth-flow/api/pkg/job"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearth-flow/api/pkg/project"
	"github.com/reearth/reearth-flow/api/pkg/trigger"
	"github.com/reearth/reearth-flow/api/pkg/variable"
	flowv1 "github.com/reearth/reearth-proto/gen/go/flow/v1"
	"google.golang.org/protobuf/types/known/structpb"
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
		CreatedBy:   p.CreatedBy().StringRef(),
		UpdatedBy:   p.UpdatedBy().StringRef(),
		IsLocked:    p.IsLocked(),
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

// triggerDeployment is a trigger's deployment and project name; either may be nil.
type triggerDeployment struct {
	deployment  *deployment.Deployment
	projectName *string
}

func toTrigger(t *trigger.Trigger, td triggerDeployment) *flowv1.Trigger {
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
	if d := td.deployment; d != nil {
		res.Deployment = toDeployment(d)
		res.ProjectName = td.projectName
		res.Variables = toTriggerVariables(d.Parameters(), t.Variables())
	}
	return res
}

// toTriggerVariables lists the deployment's variables in order, defaulting to the trigger's own values.
func toTriggerVariables(params []*parameter.Parameter, vars []variable.Variable) []*flowv1.TriggerVariable {
	own := variable.SliceToMap(vars)
	sorted := make([]*parameter.Parameter, 0, len(params))
	for _, p := range params {
		if p != nil {
			sorted = append(sorted, p)
		}
	}
	slices.SortStableFunc(sorted, func(a, b *parameter.Parameter) int { return a.Index() - b.Index() })

	res := make([]*flowv1.TriggerVariable, 0, len(sorted))
	for _, p := range sorted {
		v := &flowv1.TriggerVariable{
			Name:     p.Name(),
			Type:     toVariableType(p.Type()),
			Required: p.Required(),
		}
		def := p.DefaultValue()
		if tv, ok := own[p.Name()]; ok && tv.Value != nil {
			def = tv.Value
		}
		if def != nil {
			if pv, err := structpb.NewValue(normalizeForStruct(def)); err == nil {
				v.DefaultValue = pv
			}
		}
		if p.Type() == parameter.TypeChoice {
			v.Choices = choicesOf(p.Config())
		}
		res = append(res, v)
	}
	return res
}

// normalizeForStruct converts typed slices and maps into types structpb accepts.
func normalizeForStruct(v any) any {
	switch x := v.(type) {
	case []string:
		out := make([]any, len(x))
		for i, s := range x {
			out[i] = s
		}
		return out
	case []any:
		out := make([]any, len(x))
		for i, e := range x {
			out[i] = normalizeForStruct(e)
		}
		return out
	case map[string]any:
		out := make(map[string]any, len(x))
		for k, e := range x {
			out[k] = normalizeForStruct(e)
		}
		return out
	}
	return v
}

// choicesOf reads a choice variable's options from its config.
func choicesOf(config any) []string {
	m, ok := config.(map[string]any)
	if !ok {
		return nil
	}
	var res []string
	switch cs := m["choices"].(type) {
	case []string:
		res = append(res, cs...)
	case []any:
		for _, c := range cs {
			if s, ok := c.(string); ok {
				res = append(res, s)
			}
		}
	}
	return res
}

func toVariableType(t parameter.Type) flowv1.VariableType {
	switch t {
	case parameter.TypeText:
		return flowv1.VariableType_VARIABLE_TYPE_TEXT
	case parameter.TypeNumber:
		return flowv1.VariableType_VARIABLE_TYPE_NUMBER
	case parameter.TypeYesNo:
		return flowv1.VariableType_VARIABLE_TYPE_YES_NO
	case parameter.TypeChoice:
		return flowv1.VariableType_VARIABLE_TYPE_CHOICE
	case parameter.TypeArray:
		return flowv1.VariableType_VARIABLE_TYPE_ARRAY
	case parameter.TypeDatetime:
		return flowv1.VariableType_VARIABLE_TYPE_DATETIME
	case parameter.TypeColor:
		return flowv1.VariableType_VARIABLE_TYPE_COLOR
	case parameter.TypeFileFolder:
		return flowv1.VariableType_VARIABLE_TYPE_FILE_FOLDER
	}
	return flowv1.VariableType_VARIABLE_TYPE_UNSPECIFIED
}

func toEventSource(e trigger.EventSourceType) flowv1.TriggerEventSource {
	switch e {
	case trigger.EventSourceTypeTimeDriven:
		return flowv1.TriggerEventSource_TRIGGER_EVENT_SOURCE_TIME_DRIVEN
	case trigger.EventSourceTypeAPIDriven:
		return flowv1.TriggerEventSource_TRIGGER_EVENT_SOURCE_API_DRIVEN
	case trigger.EventSourceTypeCMS:
		return flowv1.TriggerEventSource_TRIGGER_EVENT_SOURCE_CMS
	}
	return flowv1.TriggerEventSource_TRIGGER_EVENT_SOURCE_UNSPECIFIED
}

func fromEventSource(e flowv1.TriggerEventSource) (trigger.EventSourceType, bool) {
	switch e {
	case flowv1.TriggerEventSource_TRIGGER_EVENT_SOURCE_TIME_DRIVEN:
		return trigger.EventSourceTypeTimeDriven, true
	case flowv1.TriggerEventSource_TRIGGER_EVENT_SOURCE_API_DRIVEN:
		return trigger.EventSourceTypeAPIDriven, true
	case flowv1.TriggerEventSource_TRIGGER_EVENT_SOURCE_CMS:
		return trigger.EventSourceTypeCMS, true
	}
	return "", false
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
	if j.Trigger() != nil {
		s := j.Trigger().String()
		res.TriggerId = &s
	}
	res.Origin = toJobOrigin(j.Origin())
	res.RunBy = j.RunBy().StringRef()
	for _, u := range j.OutputURLs() {
		res.Outputs = append(res.Outputs, &flowv1.JobOutput{Name: outputName(u), Url: u})
	}
	return res
}

func toJobOrigin(o *job.Origin) *flowv1.JobOrigin {
	if o == nil || o.CMSItem == nil {
		return nil
	}
	c := o.CMSItem
	return &flowv1.JobOrigin{Origin: &flowv1.JobOrigin_CmsItem{CmsItem: &flowv1.CmsItemOrigin{
		ProjectId:    c.ProjectID,
		ModelId:      c.ModelID,
		ItemId:       c.ItemID,
		ConnectionId: c.ConnectionID,
		ProjectName:  c.ProjectName,
		ModelName:    c.ModelName,
		ItemTitle:    c.ItemTitle,
	}}}
}

func fromJobOrigin(o *flowv1.JobOrigin) *job.Origin {
	c := o.GetCmsItem()
	if c == nil {
		return nil
	}
	return &job.Origin{CMSItem: &job.CMSItemOrigin{
		ProjectID:    c.GetProjectId(),
		ModelID:      c.GetModelId(),
		ItemID:       c.GetItemId(),
		ConnectionID: c.GetConnectionId(),
		ProjectName:  c.GetProjectName(),
		ModelName:    c.GetModelName(),
		ItemTitle:    c.GetItemTitle(),
	}}
}

// outputName is the file name an output URL ends in, unescaped.
func outputName(u string) string {
	p := u
	if parsed, err := url.Parse(u); err == nil {
		p = parsed.Path
	}
	name := path.Base(p)
	if unescaped, err := url.PathUnescape(name); err == nil {
		name = unescaped
	}
	return name
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
