package workflow

type NodeType struct {
	id   string
	data Data
}

type Data struct {
	name     string
	inputs   []string
	outputs  []string
	actionID string
	params   map[string]interface{}
}

func (n *NodeType) ID() string {
	return n.id
}

func (n *NodeType) Data() Data {
	return Data{
		name:     n.data.name,
		inputs:   n.data.inputs,
		outputs:  n.data.outputs,
		actionID: n.data.actionID,
		params:   n.data.params,
	}
}

func (n *NodeType) SetData(data Data) {
	n.data = data
}

func (n *NodeType) SetID(id string) {
	n.id = id
}
