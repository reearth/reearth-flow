package workflow

import "github.com/reearth/reearth-flow/api/pkg/id"

type Node struct {
	ID       id.NodeID
	Name     string
	NodeType string
	Action   string
	With     map[string]interface{}
}

func NewNode(id id.NodeID, name, nodeType, action string, with map[string]interface{}) *Node {
	return &Node{
		ID:       id,
		Name:     name,
		NodeType: nodeType,
		Action:   action,
		With:     with,
	}
}
