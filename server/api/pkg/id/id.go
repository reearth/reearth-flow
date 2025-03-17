package id

import "github.com/reearth/reearthx/idx"

type (
	Asset         struct{}
	AuthRequest   struct{}
	Deployment    struct{}
	Parameter     struct{}
	Project       struct{}
	ProjectAccess struct{}
	Trigger       struct{}
	User          struct{}
	Workflow      struct{}
	Workspace     struct{}
)

func (Asset) Type() string         { return "asset" }
func (AuthRequest) Type() string   { return "authRequest" }
func (Deployment) Type() string    { return "deployment" }
func (Parameter) Type() string     { return "parameter" }
func (Project) Type() string       { return "project" }
func (ProjectAccess) Type() string { return "projectAccess" }
func (Trigger) Type() string       { return "trigger" }
func (User) Type() string          { return "user" }
func (Workflow) Type() string      { return "workflow" }
func (Workspace) Type() string     { return "workspace" }

type (
	AssetID         = idx.ID[Asset]
	AuthRequestID   = idx.ID[AuthRequest]
	DeploymentID    = idx.ID[Deployment]
	ParameterID     = idx.ID[Parameter]
	ProjectID       = idx.ID[Project]
	ProjectAccessID = idx.ID[ProjectAccess]
	TriggerID       = idx.ID[Trigger]
	UserID          = idx.ID[User]
	WorkflowID      = idx.ID[Workflow]
	WorkspaceID     = idx.ID[Workspace]
)

var (
	NewAssetID         = idx.New[Asset]
	NewAuthRequestID   = idx.New[AuthRequest]
	NewDeploymentID    = idx.New[Deployment]
	NewParameterID     = idx.New[Parameter]
	NewProjectID       = idx.New[Project]
	NewProjectAccessID = idx.New[ProjectAccess]
	NewTriggerID       = idx.New[Trigger]
	NewUserID          = idx.New[User]
	NewWorkflowID      = idx.New[Workflow]
	NewWorkspaceID     = idx.New[Workspace]
)

var (
	MustAssetID         = idx.Must[Asset]
	MustAuthRequestID   = idx.Must[AuthRequest]
	MustDeploymentID    = idx.Must[Deployment]
	MustParameterID     = idx.Must[Parameter]
	MustProjectID       = idx.Must[Project]
	MustProjectAccessID = idx.Must[ProjectAccess]
	MustTriggerID       = idx.Must[Trigger]
	MustUserID          = idx.Must[User]
	MustWorkflowID      = idx.Must[Workflow]
	MustWorkspaceID     = idx.Must[Workspace]
)

var (
	AssetIDFrom         = idx.From[Asset]
	AuthRequestIDFrom   = idx.From[AuthRequest]
	DeploymentIDFrom    = idx.From[Deployment]
	ParameterIDFrom     = idx.From[Parameter]
	ProjectIDFrom       = idx.From[Project]
	ProjectAccessIDFrom = idx.From[ProjectAccess]
	TriggerIDFrom       = idx.From[Trigger]
	UserIDFrom          = idx.From[User]
	WorkflowIDFrom      = idx.From[Workflow]
	WorkspaceIDFrom     = idx.From[Workspace]
)

var (
	AssetIDFromRef         = idx.FromRef[Asset]
	AuthRequestIDFromRef   = idx.FromRef[AuthRequest]
	DeploymentIDFromRef    = idx.FromRef[Deployment]
	ParameterIDFromRef     = idx.FromRef[Parameter]
	ProjectIDFromRef       = idx.FromRef[Project]
	ProjectAccessIDFromRef = idx.FromRef[ProjectAccess]
	TriggerIDFromRef       = idx.FromRef[Trigger]
	UserIDFromRef          = idx.FromRef[User]
	WorkflowIDFromRef      = idx.FromRef[Workflow]
	WorkspaceIDFromRef     = idx.FromRef[Workspace]
)

type (
	AssetIDList         = idx.List[Asset]
	AuthRequestIDList   = idx.List[AuthRequest]
	DeploymentIDList    = idx.List[Deployment]
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
	NewParameterIDSet     = idx.NewSet[Parameter]
	NewProjectIDSet       = idx.NewSet[Project]
	NewProjectAccessIDSet = idx.NewSet[ProjectAccess]
	NewTriggerIDSet       = idx.NewSet[Trigger]
	NewUserIDSet          = idx.NewSet[User]
	NewWorkspaceIDSet     = idx.NewSet[Workspace]
)
