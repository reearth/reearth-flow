package parameter

import (
	"sort"

	"github.com/reearth/reearth-flow/api/pkg/id"
)

type ParameterList []*Parameter

func NewParameterList(params []*Parameter) *ParameterList {
	if params == nil {
		return nil
	}
	pl := ParameterList(params)
	return &pl
}

func (l *ParameterList) MaxIndex() int {
	if l == nil {
		return -1
	}
	max := -1
	for _, p := range *l {
		if p.Index() > max {
			max = p.Index()
		}
	}
	return max
}

func (l *ParameterList) Sort() {
	if l == nil {
		return
	}
	sort.SliceStable(*l, func(i, j int) bool {
		return (*l)[i].Index() < (*l)[j].Index()
	})
}

func (l *ParameterList) FindByID(id id.ParameterID) *Parameter {
	if l == nil {
		return nil
	}
	for _, p := range *l {
		if p.ID() == id {
			return p
		}
	}
	return nil
}

func (l *ParameterList) IDs() id.ParameterIDList {
	if l == nil {
		return nil
	}
	ids := make(id.ParameterIDList, 0, len(*l))
	for _, p := range *l {
		ids = append(ids, p.ID())
	}
	return ids
}

func (l *ParameterList) FilterByProject(pid id.ProjectID) *ParameterList {
	if l == nil {
		return nil
	}
	res := make([]*Parameter, 0, len(*l))
	for _, p := range *l {
		if p.ProjectID() == pid {
			res = append(res, p)
		}
	}
	if len(res) == 0 {
		return nil
	}
	return NewParameterList(res)
}

func (l *ParameterList) Len() int {
	if l == nil {
		return 0
	}
	return len(*l)
}

func (l *ParameterList) Clone() *ParameterList {
	if l == nil {
		return nil
	}
	res := make([]*Parameter, len(*l))
	copy(res, *l)
	return NewParameterList(res)
}
