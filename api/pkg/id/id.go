package id

import "github.com/reearth/reearthx/idx"

type Asset struct{}
type AuthRequest struct{}
type Edge struct{}
type Node struct{}
type Graph struct{}
type Workflow struct{}
type Project struct{}
type Workspace struct{}
type User struct{}
type Job struct{}
type Deployment struct{}

func (Asset) Type() string       { return "asset" }
func (AuthRequest) Type() string { return "authRequest" }
func (Edge) Type() string        { return "edge" }
func (Node) Type() string        { return "node" }
func (Graph) Type() string       { return "graph" }
func (Workflow) Type() string     { return "workflow" }
func (Project) Type() string     { return "project" }
func (Workspace) Type() string   { return "workspace" }
func (User) Type() string        { return "user" }
func (Job) Type() string         { return "job" }
func (Deployment) Type() string  { return "deployment" }

type AssetID = idx.ID[Asset]
type AuthRequestID = idx.ID[AuthRequest]
type EdgeID = idx.ID[Edge]
type NodeID = idx.ID[Node]
type GraphID = idx.ID[Graph]
type WorkflowID = idx.ID[Workflow]
type ProjectID = idx.ID[Project]
type WorkspaceID = idx.ID[Workspace]
type UserID = idx.ID[User]
type JobID = idx.ID[Job]
type DeploymentID = idx.ID[Deployment]

var NewAssetID = idx.New[Asset]
var NewAuthRequestID = idx.New[AuthRequest]
var NewEdgeID = idx.New[Edge]
var NewNodeID = idx.New[Node]
var NewGraphID = idx.New[Graph]
var NewWorkflowID = idx.New[Workflow]
var NewProjectID = idx.New[Project]
var NewJobID = idx.New[Job]
var NewDeploymentID = idx.New[Deployment]

var MustAssetID = idx.Must[Asset]
var MustAuthRequestID = idx.Must[AuthRequest]
var MustWorkflowID = idx.Must[Workflow]
var MustProjectID = idx.Must[Project]
var MustWorkspaceID = idx.Must[Workspace]
var MustUserID = idx.Must[User]
var MustJobID = idx.Must[Job]
var MustDeploymentID = idx.Must[Job]

var AssetIDFrom = idx.From[Asset]
var AuthRequestIDFrom = idx.From[AuthRequest]
var EdgeIDFrom = idx.From[Edge]
var NodeIDFrom = idx.From[Node]
var GraphIDFrom = idx.From[Graph]
var WorkflowIDFrom = idx.From[Workflow]
var ProjectIDFrom = idx.From[Project]
var WorkspaceIDFrom = idx.From[Workspace]
var UserIDFrom = idx.From[User]
var JobIDFrom = idx.From[Job]
var DeploymentIDFrom = idx.From[Deployment]

var AssetIDFromRef = idx.FromRef[Asset]
var AuthRequestIDFromRef = idx.FromRef[AuthRequest]
var WorkflowIDFromRef = idx.FromRef[Workflow]
var ProjectIDFromRef = idx.FromRef[Project]
var WorkspaceIDFromRef = idx.FromRef[Workspace]
var UserIDFromRef = idx.FromRef[User]
var JobIDFromRef = idx.FromRef[Job]
var DeploymentIDFromRef = idx.FromRef[Deployment]

type AssetIDList = idx.List[Asset]
type AuthRequestIDList = idx.List[AuthRequest]
type ProjectIDList = idx.List[Project]
type WorkspaceIDList = idx.List[Workspace]
type UserIDList = idx.List[User]
type JobIDList = idx.List[Job]
type DeploymentIDList = idx.List[Deployment]

var AssetIDListFrom = idx.ListFrom[Asset]
var AuthRequestIDListFrom = idx.ListFrom[AuthRequest]
var ProjectIDListFrom = idx.ListFrom[Project]
var WorkspaceIDListFrom = idx.ListFrom[Workspace]
var UserIDListFrom = idx.ListFrom[User]
var JobIDListFrom = idx.ListFrom[Job]
var DeploymentIDListFrom = idx.ListFrom[Deployment]

type AssetIDSet = idx.Set[Asset]
type AuthRequestIDSet = idx.Set[AuthRequest]
type ProjectIDSet = idx.Set[Project]
type WorkspaceIDSet = idx.Set[Workspace]
type UserIDSet = idx.Set[User]
type JobIDSet = idx.Set[Job]
type DeploymentIDSet = idx.Set[Deployment]

var NewAssetIDSet = idx.NewSet[Asset]
var NewAuthRequestIDSet = idx.NewSet[AuthRequest]
var NewProjectIDSet = idx.NewSet[Project]
var NewWorkspaceIDSet = idx.NewSet[Workspace]
var NewUserIDSet = idx.NewSet[User]
var NewJobIDSet = idx.NewSet[Job]
var NewDeploymentIDSet = idx.NewSet[Deployment]
