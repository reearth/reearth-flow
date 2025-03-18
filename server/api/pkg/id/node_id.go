package id

import (
	"github.com/google/uuid"
)

type Node struct{}

func (Node) Type() string { return "job" }

type NodeID struct {
	id uuid.UUID
}

func NewNodeID() NodeID {
	return NodeID{id: uuid.New()}
}

func NodeIDFrom(id string) (NodeID, error) {
	parsed, err := uuid.Parse(id)
	if err != nil {
		return NodeID{}, ErrInvalidID
	}
	return NodeID{id: parsed}, nil
}

func MustNodeID(id string) NodeID {
	jid, err := NodeIDFrom(id)
	if err != nil {
		panic(err)
	}
	return jid
}

func NodeIDFromRef(id *string) *NodeID {
	if id == nil {
		return nil
	}
	jid, err := NodeIDFrom(*id)
	if err != nil {
		return nil
	}
	return &jid
}

func (id NodeID) String() string {
	return id.id.String()
}

func (id NodeID) GoString() string {
	return "NodeID(" + id.String() + ")"
}

func (id *NodeID) IsNil() bool {
	return id == nil || id.id == uuid.Nil
}

func (id *NodeID) StringRef() *string {
	if id == nil {
		return nil
	}
	s := id.String()
	return &s
}

func (id NodeID) Ref() *NodeID {
	return &id
}

func (id NodeID) Clone() NodeID {
	return NodeID{id: id.id}
}

func (id *NodeID) CloneRef() *NodeID {
	if id == nil {
		return nil
	}
	i := id.Clone()
	return &i
}

type NodeIDList []NodeID

func NodeIDListFrom(ids []string) (NodeIDList, error) {
	if ids == nil {
		return nil, nil
	}
	list := make(NodeIDList, len(ids))
	for i, id := range ids {
		parsed, err := NodeIDFrom(id)
		if err != nil {
			return nil, err
		}
		list[i] = parsed
	}
	return list, nil
}

type NodeIDSet map[NodeID]struct{}

func NewNodeIDSet() NodeIDSet {
	return make(NodeIDSet)
}
