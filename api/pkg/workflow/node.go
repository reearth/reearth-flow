package workflow

type Node struct {
	id       string
	name     string
	nodeType string
	action   string
	with     map[string]interface{}
}

func (n *Node) ID() string {
	return n.id
}

func (n *Node) Name() string {
	return n.name
}

func (n *Node) NodeType() string {
	return n.nodeType
}

func (n *Node) Action() string {
	return n.action
}

func (n *Node) With() map[string]interface{} {
	return n.with
}

func (n *Node) SetID(id string) {
	n.id = id
}

func (n *Node) SetName(name string) {
	n.name = name
}

func (n *Node) SetNodeType(nodeType string) {
	n.nodeType = nodeType
}

func (n *Node) SetAction(action string) {
	n.action = action
}

func (n *Node) SetWith(with map[string]interface{}) {
	n.with = with
}
