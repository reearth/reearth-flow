package id

import (
	"github.com/google/uuid"
)

type Job struct{}

func (Job) Type() string { return "job" }

type JobID struct {
	id uuid.UUID
}

func NewJobID() JobID {
	return JobID{id: uuid.New()}
}

func JobIDFrom(id string) (JobID, error) {
	parsed, err := uuid.Parse(id)
	if err != nil {
		return JobID{}, ErrInvalidID
	}
	return JobID{id: parsed}, nil
}

func MustJobID(id string) JobID {
	jid, err := JobIDFrom(id)
	if err != nil {
		panic(err)
	}
	return jid
}

func JobIDFromRef(id *string) *JobID {
	if id == nil {
		return nil
	}
	jid, err := JobIDFrom(*id)
	if err != nil {
		return nil
	}
	return &jid
}

func (id JobID) String() string {
	return id.id.String()
}

func (id JobID) GoString() string {
	return "JobID(" + id.String() + ")"
}

func (id *JobID) IsNil() bool {
	return id == nil || id.id == uuid.Nil
}

func (id *JobID) StringRef() *string {
	if id == nil {
		return nil
	}
	s := id.String()
	return &s
}

func (id JobID) Ref() *JobID {
	return &id
}

func (id JobID) Clone() JobID {
	return JobID{id: id.id}
}

func (id *JobID) CloneRef() *JobID {
	if id == nil {
		return nil
	}
	i := id.Clone()
	return &i
}

type JobIDList []JobID

func JobIDListFrom(ids []string) (JobIDList, error) {
	if ids == nil {
		return nil, nil
	}
	list := make(JobIDList, len(ids))
	for i, id := range ids {
		parsed, err := JobIDFrom(id)
		if err != nil {
			return nil, err
		}
		list[i] = parsed
	}
	return list, nil
}

type JobIDSet map[JobID]struct{}

func NewJobIDSet() JobIDSet {
	return make(JobIDSet)
}
