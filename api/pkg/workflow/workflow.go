package workflow

type Workflow struct {
	id             ID
	nodes          []NodeType
	edges          []Edges
	isMain         bool
	projectVersion int
	projectID      ProjectID
	workspaceID    WorkspaceID
}

func (w *Workflow) ID() ID {
	return w.id
}

func (w *Workflow) Nodes() []NodeType {
	return w.nodes
}

func (w *Workflow) Edges() []Edges {
	return w.edges
}

func (w *Workflow) IsMain() bool {
	return w.isMain
}

func (w *Workflow) ProjectVersion() int {
	return w.projectVersion
}

func (w *Workflow) ProjectID() ProjectID {
	return w.projectID
}

func (w *Workflow) WorkspaceID() WorkspaceID {
	return w.workspaceID
}

func (w *Workflow) SetID(id ID) {
	w.id = id
}

func (w *Workflow) SetNodes(nodes []NodeType) {
	w.nodes = nodes
}

func (w *Workflow) SetEdges(edges []Edges) {
	w.edges = edges
}

func (w *Workflow) SetIsMain(isMain bool) {
	w.isMain = isMain
}

func (w *Workflow) SetProjectVersion(projectVersion int) {
	w.projectVersion = projectVersion
}

func (w *Workflow) SetProjectID(projectID ProjectID) {
	w.projectID = projectID
}

func (w *Workflow) SetWorkspaceID(workspaceID WorkspaceID) {
	w.workspaceID = workspaceID
}
