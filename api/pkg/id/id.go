package id

import "github.com/reearth/reearthx/idx"

type (
	Asset       struct{}
	AuthRequest struct{}
	Edge        struct{}
	Node        struct{}
	Graph       struct{}
	Workflow    struct{}
	Project     struct{}
	Workspace   struct{}
	User        struct{}
	Job         struct{}
	Deployment  struct{}
)

func (Asset) Type() string       { return "asset" }
func (AuthRequest) Type() string { return "authRequest" }
func (Edge) Type() string        { return "edge" }
func (Node) Type() string        { return "node" }
func (Graph) Type() string       { return "graph" }
func (Workflow) Type() string    { return "workflow" }
func (Project) Type() string     { return "project" }
func (Workspace) Type() string   { return "workspace" }
func (User) Type() string        { return "user" }
func (Job) Type() string         { return "job" }
func (Deployment) Type() string  { return "deployment" }

type (
	AssetID       = idx.ID[Asset]
	AuthRequestID = idx.ID[AuthRequest]
	EdgeID        = idx.ID[Edge]
	NodeID        = idx.ID[Node]
	GraphID       = idx.ID[Graph]
	WorkflowID    = idx.ID[Workflow]
	ProjectID     = idx.ID[Project]
	WorkspaceID   = idx.ID[Workspace]
	UserID        = idx.ID[User]
	JobID         = idx.ID[Job]
	DeploymentID  = idx.ID[Deployment]
)

var (
	NewAssetID       = idx.New[Asset]
	NewAuthRequestID = idx.New[AuthRequest]
	NewEdgeID        = idx.New[Edge]
	NewNodeID        = idx.New[Node]
	NewGraphID       = idx.New[Graph]
	NewWorkflowID    = idx.New[Workflow]
	NewProjectID     = idx.New[Project]
	NewJobID         = idx.New[Job]
	NewDeploymentID  = idx.New[Deployment]
)

var (
	MustAssetID       = idx.Must[Asset]
	MustAuthRequestID = idx.Must[AuthRequest]
	MustWorkflowID    = idx.Must[Workflow]
	MustProjectID     = idx.Must[Project]
	MustWorkspaceID   = idx.Must[Workspace]
	MustUserID        = idx.Must[User]
	MustJobID         = idx.Must[Job]
	MustDeploymentID  = idx.Must[Job]
)

var (
	AssetIDFrom       = idx.From[Asset]
	AuthRequestIDFrom = idx.From[AuthRequest]
	EdgeIDFrom        = idx.From[Edge]
	NodeIDFrom        = idx.From[Node]
	GraphIDFrom       = idx.From[Graph]
	WorkflowIDFrom    = idx.From[Workflow]
	ProjectIDFrom     = idx.From[Project]
	WorkspaceIDFrom   = idx.From[Workspace]
	UserIDFrom        = idx.From[User]
	JobIDFrom         = idx.From[Job]
	DeploymentIDFrom  = idx.From[Deployment]
)

var (
	AssetIDFromRef       = idx.FromRef[Asset]
	AuthRequestIDFromRef = idx.FromRef[AuthRequest]
	WorkflowIDFromRef    = idx.FromRef[Workflow]
	ProjectIDFromRef     = idx.FromRef[Project]
	WorkspaceIDFromRef   = idx.FromRef[Workspace]
	UserIDFromRef        = idx.FromRef[User]
	JobIDFromRef         = idx.FromRef[Job]
	DeploymentIDFromRef  = idx.FromRef[Deployment]
)

type (
	AssetIDList       = idx.List[Asset]
	AuthRequestIDList = idx.List[AuthRequest]
	ProjectIDList     = idx.List[Project]
	WorkspaceIDList   = idx.List[Workspace]
	UserIDList        = idx.List[User]
	JobIDList         = idx.List[Job]
	DeploymentIDList  = idx.List[Deployment]
)

var (
	AssetIDListFrom       = idx.ListFrom[Asset]
	AuthRequestIDListFrom = idx.ListFrom[AuthRequest]
	ProjectIDListFrom     = idx.ListFrom[Project]
	WorkspaceIDListFrom   = idx.ListFrom[Workspace]
	UserIDListFrom        = idx.ListFrom[User]
	JobIDListFrom         = idx.ListFrom[Job]
	DeploymentIDListFrom  = idx.ListFrom[Deployment]
)

type (
	AssetIDSet       = idx.Set[Asset]
	AuthRequestIDSet = idx.Set[AuthRequest]
	ProjectIDSet     = idx.Set[Project]
	WorkspaceIDSet   = idx.Set[Workspace]
	UserIDSet        = idx.Set[User]
	JobIDSet         = idx.Set[Job]
	DeploymentIDSet  = idx.Set[Deployment]
)

var (
	NewAssetIDSet       = idx.NewSet[Asset]
	NewAuthRequestIDSet = idx.NewSet[AuthRequest]
	NewProjectIDSet     = idx.NewSet[Project]
	NewWorkspaceIDSet   = idx.NewSet[Workspace]
	NewUserIDSet        = idx.NewSet[User]
	NewJobIDSet         = idx.NewSet[Job]
	NewDeploymentIDSet  = idx.NewSet[Deployment]
)
