package workflow

import "github.com/reearth/reearth-flow/api/pkg/id"

type Node struct {
	id       id.NodeID
	name     string
	nodeType string
	action   string
	with     map[string]interface{}
}

func NewNode(id id.NodeID, name, nodeType, action string, with map[string]interface{}) *Node {
	return &Node{
		id:       id,
		name:     name,
		nodeType: nodeType,
		action:   action,
		with:     with,
	}
}

func (n *Node) ID() id.NodeID {
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
