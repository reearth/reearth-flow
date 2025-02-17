package gql

import (
	"context"

	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/document"
)

func (r *queryResolver) DocumentLatest(ctx context.Context, id gqlmodel.ID) (*gqlmodel.Document, error) {
	doc, err := document.GetLatest(ctx, string(id))
	if err != nil {
		return nil, err
	}

	return &gqlmodel.Document{
		ID:        id,
		Update:    doc.Update,
		Clock:     doc.Clock,
		Timestamp: doc.Timestamp,
	}, nil
}

func (r *queryResolver) DocumentSnapshot(ctx context.Context, id gqlmodel.ID) ([]*gqlmodel.DocumentSnapshot, error) {
	history, err := document.GetHistory(ctx, string(id))
	if err != nil {
		return nil, err
	}

	nodes := make([]*gqlmodel.DocumentSnapshot, len(history))
	for i, h := range history {
		nodes[i] = &gqlmodel.DocumentSnapshot{
			Update:    h.Update,
			Clock:     h.Clock,
			Timestamp: h.Timestamp,
		}
	}

	return nodes, nil
}

func (r *mutationResolver) DocumentRollback(ctx context.Context, id gqlmodel.ID, clock int) (*gqlmodel.Document, error) {
	doc, err := document.Rollback(ctx, string(id), clock)
	if err != nil {
		return nil, err
	}

	return &gqlmodel.Document{
		ID:        id,
		Update:    doc.Update,
		Clock:     doc.Clock,
		Timestamp: doc.Timestamp,
	}, nil
}

type documentResolver struct{ *Resolver }

func (r *Resolver) Document() DocumentResolver {
	return &documentResolver{r}
}

func (r *documentResolver) Update(ctx context.Context, obj *gqlmodel.Document) ([]int, error) {
	return obj.Update, nil
}
