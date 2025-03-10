package gql

import (
	"context"
	
	"github.com/reearth/reearth-flow/api/internal/adapter/gql/gqlmodel"
	"github.com/reearth/reearth-flow/api/internal/usecase/interactor"
	"github.com/reearth/reearthx/log"
)

func (r *queryResolver) LatestDocument(ctx context.Context, input gqlmodel.GetLatestDocumentInput) (*gqlmodel.Document, error) {
	log.Debugfc(ctx, "gql: fetching latest document for projectId=%s", input.ProjectID)
	
	doc, err := interactor.GetLatest(ctx, string(input.ProjectID))
	if err != nil {
		log.Errorfc(ctx, "gql: failed to get latest document: %v", err)
		return nil, err
	}
	
	result := &gqlmodel.Document{
		ID:        input.ProjectID,
		Updates:   doc.Updates,
		Version:   doc.Version,
		Timestamp: doc.Timestamp,
	}
	
	log.Debugfc(ctx, "gql: successfully retrieved latest document with version=%d", doc.Version)
	return result, nil
}

func (r *queryResolver) DocumentHistory(ctx context.Context, input gqlmodel.GetDocumentHistoryInput) ([]*gqlmodel.DocumentSnapshot, error) {
	log.Debugfc(ctx, "gql: fetching document history for projectId=%s", input.ProjectID)
	
	history, err := interactor.GetHistory(ctx, string(input.ProjectID))
	if err != nil {
		log.Errorfc(ctx, "gql: failed to get document history: %v", err)
		return nil, err
	}
	
	nodes := make([]*gqlmodel.DocumentSnapshot, len(history))
	for i, h := range history {
		nodes[i] = &gqlmodel.DocumentSnapshot{
			Updates:   h.Updates,
			Version:   h.Version,
			Timestamp: h.Timestamp,
		}
	}
	
	log.Debugfc(ctx, "gql: successfully retrieved %d document history entries", len(nodes))
	return nodes, nil
}

func (r *mutationResolver) RollbackDocument(ctx context.Context, input gqlmodel.RollbackDocumentInput) (*gqlmodel.DocumentPayload, error) {
	log.Debugfc(ctx, "gql: rolling back document projectId=%s to version=%d", input.ProjectID, input.Version)
	
	doc, err := interactor.Rollback(ctx, string(input.ProjectID), input.Version)
	if err != nil {
		log.Errorfc(ctx, "gql: failed to rollback document: %v", err)
		return nil, err
	}
	
	result := &gqlmodel.Document{
		ID:        input.ProjectID,
		Updates:   doc.Updates,
		Version:   doc.Version,
		Timestamp: doc.Timestamp,
	}
	
	log.Debugfc(ctx, "gql: successfully rolled back document to version=%d", doc.Version)
	return &gqlmodel.DocumentPayload{Document: result}, nil
}

type documentResolver struct{ *Resolver }

func (r *Resolver) Document() DocumentResolver {
	return &documentResolver{r}
}

func (r *documentResolver) Updates(ctx context.Context, obj *gqlmodel.Document) ([]int, error) {
	return obj.Updates, nil
}
