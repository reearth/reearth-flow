package id

import (
	"github.com/google/uuid"
)

type Edge struct{}

func (Edge) Type() string { return "job" }

type EdgeID struct {
	id uuid.UUID
}

func NewEdgeID() EdgeID {
	return EdgeID{id: uuid.New()}
}

func EdgeIDFrom(id string) (EdgeID, error) {
	parsed, err := uuid.Parse(id)
	if err != nil {
		return EdgeID{}, ErrInvalidID
	}
	return EdgeID{id: parsed}, nil
}

func MustEdgeID(id string) EdgeID {
	jid, err := EdgeIDFrom(id)
	if err != nil {
		panic(err)
	}
	return jid
}

func EdgeIDFromRef(id *string) *EdgeID {
	if id == nil {
		return nil
	}
	jid, err := EdgeIDFrom(*id)
	if err != nil {
		return nil
	}
	return &jid
}

func (id EdgeID) String() string {
	return id.id.String()
}

func (id EdgeID) GoString() string {
	return "EdgeID(" + id.String() + ")"
}

func (id *EdgeID) IsNil() bool {
	return id == nil || id.id == uuid.Nil
}

func (id *EdgeID) StringRef() *string {
	if id == nil {
		return nil
	}
	s := id.String()
	return &s
}

func (id EdgeID) Ref() *EdgeID {
	return &id
}

func (id EdgeID) Clone() EdgeID {
	return EdgeID{id: id.id}
}

func (id *EdgeID) CloneRef() *EdgeID {
	if id == nil {
		return nil
	}
	i := id.Clone()
	return &i
}

type EdgeIDList []EdgeID

func EdgeIDListFrom(ids []string) (EdgeIDList, error) {
	if ids == nil {
		return nil, nil
	}
	list := make(EdgeIDList, len(ids))
	for i, id := range ids {
		parsed, err := EdgeIDFrom(id)
		if err != nil {
			return nil, err
		}
		list[i] = parsed
	}
	return list, nil
}

type EdgeIDSet map[EdgeID]struct{}

func NewEdgeIDSet() EdgeIDSet {
	return make(EdgeIDSet)
}
