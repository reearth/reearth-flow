package memory

import (
	"context"
	"sync"

	"github.com/reearth/reearth-flow/api/internal/usecase/repo"
	"github.com/reearth/reearth-flow/api/pkg/id"
	"github.com/reearth/reearth-flow/api/pkg/parameter"
	"github.com/reearth/reearthx/rerror"
)

type Parameter struct {
	data map[id.ParameterID]*parameter.Parameter
	lock sync.Mutex
}

func NewParameter() *Parameter {
	return &Parameter{
		data: map[id.ParameterID]*parameter.Parameter{},
	}
}

func NewParameterWith(items ...*parameter.Parameter) repo.Parameter {
	r := NewParameter()
	ctx := context.Background()
	for _, i := range items {
		_ = r.Save(ctx, i)
	}
	return r
}

func (r *Parameter) FindByID(ctx context.Context, id id.ParameterID) (*parameter.Parameter, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	res, ok := r.data[id]
	if !ok {
		return nil, rerror.ErrNotFound
	}
	return res, nil
}

func (r *Parameter) FindByIDs(ctx context.Context, ids id.ParameterIDList) (*parameter.ParameterList, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	result := make([]*parameter.Parameter, 0, len(ids))
	for _, id := range ids {
		if d, ok := r.data[id]; ok {
			result = append(result, d)
		}
	}
	return parameter.NewParameterList(result), nil
}

func (r *Parameter) FindByProject(ctx context.Context, projectID id.ProjectID) (*parameter.ParameterList, error) {
	r.lock.Lock()
	defer r.lock.Unlock()

	result := make([]*parameter.Parameter, 0)
	for _, p := range r.data {
		if p.ProjectID() == projectID {
			result = append(result, p)
		}
	}

	pl := parameter.NewParameterList(result)
	if pl != nil {
		pl.Sort() // Sort by index
	}

	return pl, nil
}

func (r *Parameter) Save(ctx context.Context, p *parameter.Parameter) error {
	r.lock.Lock()
	defer r.lock.Unlock()

	r.data[p.ID()] = p
	return nil
}

func (r *Parameter) Remove(ctx context.Context, id id.ParameterID) error {
	r.lock.Lock()
	defer r.lock.Unlock()

	if _, ok := r.data[id]; !ok {
		return rerror.ErrNotFound
	}

	delete(r.data, id)
	return nil
}

func (r *Parameter) RemoveAll(ctx context.Context, ids id.ParameterIDList) error {
	r.lock.Lock()
	defer r.lock.Unlock()

	for _, id := range ids {
		delete(r.data, id)
	}
	return nil
}

func (r *Parameter) RemoveAllByProject(ctx context.Context, projectID id.ProjectID) error {
	r.lock.Lock()
	defer r.lock.Unlock()

	for pid, p := range r.data {
		if p.ProjectID() == projectID {
			delete(r.data, pid)
		}
	}
	return nil
}
