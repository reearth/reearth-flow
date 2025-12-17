package id

import "github.com/reearth/reearthx/idx"

type (
	Asset         struct{}
	AuthRequest   struct{}
	Deployment    struct{}
	EdgeExecution struct{}
	Integration   struct{}
	NodeExecution struct{}
	Parameter     struct{}
	Project       struct{}
	ProjectAccess struct{}
	Thread        struct{}
	Trigger       struct{}
	User          struct{}
	Workflow      struct{}
	WorkerConfig  struct{}
	Workspace     struct{}
)

func (Asset) Type() string         { return "asset" }
func (AuthRequest) Type() string   { return "authRequest" }
func (Deployment) Type() string    { return "deployment" }
func (EdgeExecution) Type() string { return "edgeExecution" }
func (Integration) Type() string   { return "integration" }
func (NodeExecution) Type() string { return "nodeExecution" }
func (Parameter) Type() string     { return "parameter" }
func (Project) Type() string       { return "project" }
func (ProjectAccess) Type() string { return "projectAccess" }
func (Thread) Type() string        { return "thread" }
func (Trigger) Type() string       { return "trigger" }
func (User) Type() string          { return "user" }
func (Workflow) Type() string      { return "workflow" }
func (WorkerConfig) Type() string  { return "workerConfig" }
func (Workspace) Type() string     { return "workspace" }

type (
	AssetID         = idx.ID[Asset]
	AuthRequestID   = idx.ID[AuthRequest]
	DeploymentID    = idx.ID[Deployment]
	EdgeExecutionID = idx.ID[EdgeExecution]
	IntegrationID   = idx.ID[Integration]
	NodeExecutionID = idx.ID[NodeExecution]
	ParameterID     = idx.ID[Parameter]
	ProjectID       = idx.ID[Project]
	ProjectAccessID = idx.ID[ProjectAccess]
	ThreadID        = idx.ID[Thread]
	TriggerID       = idx.ID[Trigger]
	UserID          = idx.ID[User]
	WorkflowID      = idx.ID[Workflow]
	WorkerConfigID  = idx.ID[WorkerConfig]
	WorkspaceID     = idx.ID[Workspace]
)

var (
	NewAssetID         = idx.New[Asset]
	NewAuthRequestID   = idx.New[AuthRequest]
	NewDeploymentID    = idx.New[Deployment]
	NewEdgeExecutionID = idx.New[EdgeExecution]
	NewIntegrationID   = idx.New[Integration]
	NewNodeExecutionID = idx.New[NodeExecution]
	NewParameterID     = idx.New[Parameter]
	NewProjectID       = idx.New[Project]
	NewProjectAccessID = idx.New[ProjectAccess]
	NewThreadID        = idx.New[Thread]
	NewTriggerID       = idx.New[Trigger]
	NewUserID          = idx.New[User]
	NewWorkflowID      = idx.New[Workflow]
	NewWorkerConfigID  = idx.New[WorkerConfig]
	NewWorkspaceID     = idx.New[Workspace]
)

var (
	MustAssetID         = idx.Must[Asset]
	MustAuthRequestID   = idx.Must[AuthRequest]
	MustDeploymentID    = idx.Must[Deployment]
	MustEdgeExecutionID = idx.Must[EdgeExecution]
	MustIntegrationID   = idx.Must[Integration]
	MustNodeExecutionID = idx.Must[NodeExecution]
	MustParameterID     = idx.Must[Parameter]
	MustProjectID       = idx.Must[Project]
	MustProjectAccessID = idx.Must[ProjectAccess]
	MustThreadID        = idx.Must[Thread]
	MustTriggerID       = idx.Must[Trigger]
	MustUserID          = idx.Must[User]
	MustWorkflowID      = idx.Must[Workflow]
	MustWorkerConfigID  = idx.Must[WorkerConfig]
	MustWorkspaceID     = idx.Must[Workspace]
)

var (
	AssetIDFrom         = idx.From[Asset]
	AuthRequestIDFrom   = idx.From[AuthRequest]
	DeploymentIDFrom    = idx.From[Deployment]
	EdgeExecutionIDFrom = idx.From[EdgeExecution]
	IntegrationIDFrom   = idx.From[Integration]
	NodeExecutionIDFrom = idx.From[NodeExecution]
	ParameterIDFrom     = idx.From[Parameter]
	ProjectIDFrom       = idx.From[Project]
	ProjectAccessIDFrom = idx.From[ProjectAccess]
	ThreadIDFrom        = idx.From[Thread]
	TriggerIDFrom       = idx.From[Trigger]
	UserIDFrom          = idx.From[User]
	WorkflowIDFrom      = idx.From[Workflow]
	WorkerConfigIDFrom  = idx.From[WorkerConfig]
	WorkspaceIDFrom     = idx.From[Workspace]
)

var (
	AssetIDFromRef         = idx.FromRef[Asset]
	AuthRequestIDFromRef   = idx.FromRef[AuthRequest]
	DeploymentIDFromRef    = idx.FromRef[Deployment]
	EdgeExecutionIDFromRef = idx.FromRef[EdgeExecution]
	IntegrationIDFromRef   = idx.FromRef[Integration]
	NodeExecutionIDFromRef = idx.FromRef[NodeExecution]
	ParameterIDFromRef     = idx.FromRef[Parameter]
	ProjectIDFromRef       = idx.FromRef[Project]
	ProjectAccessIDFromRef = idx.FromRef[ProjectAccess]
	ThreadIDFromRef        = idx.FromRef[Thread]
	TriggerIDFromRef       = idx.FromRef[Trigger]
	UserIDFromRef          = idx.FromRef[User]
	WorkflowIDFromRef      = idx.FromRef[Workflow]
	WorkerConfigIDFromRef  = idx.FromRef[WorkerConfig]
	WorkspaceIDFromRef     = idx.FromRef[Workspace]
)

type (
	AssetIDList         = idx.List[Asset]
	AuthRequestIDList   = idx.List[AuthRequest]
	DeploymentIDList    = idx.List[Deployment]
	EdgeExecutionIDList = idx.List[EdgeExecution]
	NodeExecutionIDList = idx.List[NodeExecution]
	ParameterIDList     = idx.List[Parameter]
	ProjectIDList       = idx.List[Project]
	ProjectAccessIDList = idx.List[ProjectAccess]
	TriggerIDList       = idx.List[Trigger]
	UserIDList          = idx.List[User]
	WorkspaceIDList     = idx.List[Workspace]
)

var (
	AssetIDListFrom         = idx.ListFrom[Asset]
	AuthRequestIDListFrom   = idx.ListFrom[AuthRequest]
	DeploymentIDListFrom    = idx.ListFrom[Deployment]
	EdgeExecutionIDListFrom = idx.ListFrom[EdgeExecution]
	NodeExecutionIDListFrom = idx.ListFrom[NodeExecution]
	ParameterIDListFrom     = idx.ListFrom[Parameter]
	ProjectIDListFrom       = idx.ListFrom[Project]
	ProjectAccessIDListFrom = idx.ListFrom[ProjectAccess]
	TriggerIDListFrom       = idx.ListFrom[Trigger]
	UserIDListFrom          = idx.ListFrom[User]
	WorkspaceIDListFrom     = idx.ListFrom[Workspace]
)

type (
	AssetIDSet         = idx.Set[Asset]
	AuthRequestIDSet   = idx.Set[AuthRequest]
	DeploymentIDSet    = idx.Set[Deployment]
	EdgeExecutionIDSet = idx.Set[EdgeExecution]
	NodeExecutionIDSet = idx.Set[NodeExecution]
	ParameterIDSet     = idx.Set[Parameter]
	ProjectIDSet       = idx.Set[Project]
	ProjectAccessIDSet = idx.Set[ProjectAccess]
	TriggerIDSet       = idx.Set[Trigger]
	UserIDSet          = idx.Set[User]
	WorkspaceIDSet     = idx.Set[Workspace]
)

var (
	NewAssetIDSet         = idx.NewSet[Asset]
	NewAuthRequestIDSet   = idx.NewSet[AuthRequest]
	NewDeploymentIDSet    = idx.NewSet[Deployment]
	NewEdgeExecutionIDSet = idx.NewSet[EdgeExecution]
	NewNodeExecutionIDSet = idx.NewSet[NodeExecution]
	NewParameterIDSet     = idx.NewSet[Parameter]
	NewProjectIDSet       = idx.NewSet[Project]
	NewProjectAccessIDSet = idx.NewSet[ProjectAccess]
	NewTriggerIDSet       = idx.NewSet[Trigger]
	NewUserIDSet          = idx.NewSet[User]
	NewWorkspaceIDSet     = idx.NewSet[Workspace]
)
