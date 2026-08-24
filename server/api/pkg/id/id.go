package id

import "github.com/reearth/reearthx/idx"

type (
	Asset         struct{}
	AuthRequest   struct{}
	Deployment    struct{}
	Integration   struct{}
	Parameter     struct{}
	Project       struct{}
	ProjectAccess struct{}
	Thread        struct{}
	Trigger       struct{}
	Workflow      struct{}
	WorkerConfig  struct{}
)

func (Asset) Type() string         { return "asset" }
func (AuthRequest) Type() string   { return "authRequest" }
func (Deployment) Type() string    { return "deployment" }
func (Integration) Type() string   { return "integration" }
func (Parameter) Type() string     { return "parameter" }
func (Project) Type() string       { return "project" }
func (ProjectAccess) Type() string { return "projectAccess" }
func (Thread) Type() string        { return "thread" }
func (Trigger) Type() string       { return "trigger" }
func (Workflow) Type() string      { return "workflow" }
func (WorkerConfig) Type() string  { return "workerConfig" }

type (
	AssetID         = idx.ID[Asset]
	AuthRequestID   = idx.ID[AuthRequest]
	DeploymentID    = idx.ID[Deployment]
	IntegrationID   = idx.ID[Integration]
	ParameterID     = idx.ID[Parameter]
	ProjectID       = idx.ID[Project]
	ProjectAccessID = idx.ID[ProjectAccess]
	ThreadID        = idx.ID[Thread]
	TriggerID       = idx.ID[Trigger]
	WorkflowID      = idx.ID[Workflow]
	WorkerConfigID  = idx.ID[WorkerConfig]
)

var (
	NewAssetID         = idx.New[Asset]
	NewAuthRequestID   = idx.New[AuthRequest]
	NewDeploymentID    = idx.New[Deployment]
	NewIntegrationID   = idx.New[Integration]
	NewParameterID     = idx.New[Parameter]
	NewProjectID       = idx.New[Project]
	NewProjectAccessID = idx.New[ProjectAccess]
	NewThreadID        = idx.New[Thread]
	NewTriggerID       = idx.New[Trigger]
	NewWorkflowID      = idx.New[Workflow]
	NewWorkerConfigID  = idx.New[WorkerConfig]
)

var (
	MustAssetID         = idx.Must[Asset]
	MustAuthRequestID   = idx.Must[AuthRequest]
	MustDeploymentID    = idx.Must[Deployment]
	MustIntegrationID   = idx.Must[Integration]
	MustParameterID     = idx.Must[Parameter]
	MustProjectID       = idx.Must[Project]
	MustProjectAccessID = idx.Must[ProjectAccess]
	MustThreadID        = idx.Must[Thread]
	MustTriggerID       = idx.Must[Trigger]
	MustWorkflowID      = idx.Must[Workflow]
	MustWorkerConfigID  = idx.Must[WorkerConfig]
)

var (
	AssetIDFrom         = idx.From[Asset]
	AuthRequestIDFrom   = idx.From[AuthRequest]
	DeploymentIDFrom    = idx.From[Deployment]
	IntegrationIDFrom   = idx.From[Integration]
	ParameterIDFrom     = idx.From[Parameter]
	ProjectIDFrom       = idx.From[Project]
	ProjectAccessIDFrom = idx.From[ProjectAccess]
	ThreadIDFrom        = idx.From[Thread]
	TriggerIDFrom       = idx.From[Trigger]
	WorkflowIDFrom      = idx.From[Workflow]
	WorkerConfigIDFrom  = idx.From[WorkerConfig]
)

var (
	AssetIDFromRef         = idx.FromRef[Asset]
	AuthRequestIDFromRef   = idx.FromRef[AuthRequest]
	DeploymentIDFromRef    = idx.FromRef[Deployment]
	IntegrationIDFromRef   = idx.FromRef[Integration]
	ParameterIDFromRef     = idx.FromRef[Parameter]
	ProjectIDFromRef       = idx.FromRef[Project]
	ProjectAccessIDFromRef = idx.FromRef[ProjectAccess]
	ThreadIDFromRef        = idx.FromRef[Thread]
	TriggerIDFromRef       = idx.FromRef[Trigger]
	WorkflowIDFromRef      = idx.FromRef[Workflow]
	WorkerConfigIDFromRef  = idx.FromRef[WorkerConfig]
)

type (
	AssetIDList         = idx.List[Asset]
	AuthRequestIDList   = idx.List[AuthRequest]
	DeploymentIDList    = idx.List[Deployment]
	ParameterIDList     = idx.List[Parameter]
	ProjectIDList       = idx.List[Project]
	ProjectAccessIDList = idx.List[ProjectAccess]
	TriggerIDList       = idx.List[Trigger]
)

var (
	AssetIDListFrom         = idx.ListFrom[Asset]
	AuthRequestIDListFrom   = idx.ListFrom[AuthRequest]
	DeploymentIDListFrom    = idx.ListFrom[Deployment]
	ParameterIDListFrom     = idx.ListFrom[Parameter]
	ProjectIDListFrom       = idx.ListFrom[Project]
	ProjectAccessIDListFrom = idx.ListFrom[ProjectAccess]
	TriggerIDListFrom       = idx.ListFrom[Trigger]
)

type (
	AssetIDSet         = idx.Set[Asset]
	AuthRequestIDSet   = idx.Set[AuthRequest]
	DeploymentIDSet    = idx.Set[Deployment]
	ParameterIDSet     = idx.Set[Parameter]
	ProjectIDSet       = idx.Set[Project]
	ProjectAccessIDSet = idx.Set[ProjectAccess]
	TriggerIDSet       = idx.Set[Trigger]
)

var (
	NewAssetIDSet         = idx.NewSet[Asset]
	NewAuthRequestIDSet   = idx.NewSet[AuthRequest]
	NewDeploymentIDSet    = idx.NewSet[Deployment]
	NewParameterIDSet     = idx.NewSet[Parameter]
	NewProjectIDSet       = idx.NewSet[Project]
	NewProjectAccessIDSet = idx.NewSet[ProjectAccess]
	NewTriggerIDSet       = idx.NewSet[Trigger]
)
