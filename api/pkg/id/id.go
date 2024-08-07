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

func (Asset) Type() string       { return "asset" }
func (AuthRequest) Type() string { return "authRequest" }
func (Edge) Type() string        { return "edge" }
func (Node) Type() string        { return "node" }
func (Graph) Type() string       { return "graph" }
func (Workflow) Type() string    { return "workflow" }
func (Project) Type() string     { return "project" }
func (Workspace) Type() string   { return "workspace" }
func (User) Type() string        { return "user" }

type AssetID = idx.ID[Asset]
type AuthRequestID = idx.ID[AuthRequest]
type EdgeID = idx.ID[Edge]
type NodeID = idx.ID[Node]
type GraphID = idx.ID[Graph]
type WorkflowID = idx.ID[Workflow]
type ProjectID = idx.ID[Project]
type WorkspaceID = idx.ID[Workspace]
type UserID = idx.ID[User]

var NewAssetID = idx.New[Asset]
var NewAuthRequestID = idx.New[AuthRequest]
var NewEdgeID = idx.New[Edge]
var NewNodeID = idx.New[Node]
var NewGraphID = idx.New[Graph]
var NewWorkflowID = idx.New[Workflow]
var NewProjectID = idx.New[Project]

var MustAssetID = idx.Must[Asset]
var MustAuthRequestID = idx.Must[AuthRequest]
var MustWorkflowID = idx.Must[Workflow]
var MustProjectID = idx.Must[Project]
var MustWorkspaceID = idx.Must[Workspace]
var MustUserID = idx.Must[User]

var AssetIDFrom = idx.From[Asset]
var AuthRequestIDFrom = idx.From[AuthRequest]
var EdgeIDFrom = idx.From[Edge]
var NodeIDFrom = idx.From[Node]
var GraphIDFrom = idx.From[Graph]
var WorkflowIDFrom = idx.From[Workflow]
var ProjectIDFrom = idx.From[Project]
var WorkspaceIDFrom = idx.From[Workspace]
var UserIDFrom = idx.From[User]

var AssetIDFromRef = idx.FromRef[Asset]
var AuthRequestIDFromRef = idx.FromRef[AuthRequest]
var WorkflowIDFromRef = idx.FromRef[Workflow]
var ProjectIDFromRef = idx.FromRef[Project]
var WorkspaceIDFromRef = idx.FromRef[Workspace]
var UserIDFromRef = idx.FromRef[User]

type AssetIDList = idx.List[Asset]
type AuthRequestIDList = idx.List[AuthRequest]
type ProjectIDList = idx.List[Project]
type WorkspaceIDList = idx.List[Workspace]
type UserIDList = idx.List[User]

var AssetIDListFrom = idx.ListFrom[Asset]
var AuthRequestIDListFrom = idx.ListFrom[AuthRequest]
var ProjectIDListFrom = idx.ListFrom[Project]
var WorkspaceIDListFrom = idx.ListFrom[Workspace]
var UserIDListFrom = idx.ListFrom[User]

type AssetIDSet = idx.Set[Asset]
type AuthRequestIDSet = idx.Set[AuthRequest]
type ProjectIDSet = idx.Set[Project]
type WorkspaceIDSet = idx.Set[Workspace]
type UserIDSet = idx.Set[User]

var NewAssetIDSet = idx.NewSet[Asset]
var NewAuthRequestIDSet = idx.NewSet[AuthRequest]
var NewProjectIDSet = idx.NewSet[Project]
var NewWorkspaceIDSet = idx.NewSet[Workspace]
var NewUserIDSet = idx.NewSet[User]
