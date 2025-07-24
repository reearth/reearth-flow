package main

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/reearthcms"
	cmspkg "github.com/reearth/reearth-flow/api/pkg/cms"
)

func main() {
	// Configuration
	endpoint := "grpc.cms.dev.reearth.io:443"
	token := "fuewiqhriiu38475y42fd"
	workspaceID := "01jy5pem6swjmkj7q6zfbgzxk5"
	projectID := "01k06ybhb4km5s8cpe0c93xeda"

	client, err := reearthcms.NewClient(endpoint, token, true)
	if err != nil {
		fmt.Printf("Error creating client: %v\n", err)
		return
	}

	ctx := context.Background()

	// Test GetProject
	resp, err := client.GetProject(ctx, "test-project")
	if err != nil {
		fmt.Printf("GetProject error: %v\n", err)
	} else {
		fmt.Printf("GetProject success: %v\n", resp)
	}

	// Test ListProjects
	projects, totalCount, err := client.ListProjects(ctx, cmspkg.ListProjectsInput{
		WorkspaceID: workspaceID,
		PublicOnly:  true,
	})
	if err != nil {
		fmt.Printf("ListProjects error: %v\n", err)
	} else {
		fmt.Printf("ListProjects success: %v\n", projects)
		fmt.Printf("Total count: %d\n", totalCount)
	}

	// Test ListModels
	modelsResp, totalCount, err := client.ListModels(ctx, cmspkg.ListModelsInput{
		ProjectID: projectID,
	})
	if err != nil {
		fmt.Printf("ListModels error: %v\n", err)
	} else {
		fmt.Printf("ListModels success: %v\n", modelsResp)
		fmt.Printf("Total count: %d\n", totalCount)
	}

	// Test GetModelGeoJSONExportURL
	exportResp, err := client.GetModelGeoJSONExportURL(ctx, cmspkg.ExportInput{
		ProjectID: projectID,
		ModelID:   "test-model-id",
	})
	if err != nil {
		fmt.Printf("GetModelGeoJSONExportURL error: %v\n", err)
	} else {
		fmt.Printf("GetModelGeoJSONExportURL success: %v\n", exportResp)
	}

	// Test ListItems
	page := int32(1)
	pageSize := int32(10)
	itemsResp, err := client.ListItems(ctx, cmspkg.ListItemsInput{
		ProjectID: projectID,
		ModelID:   "test-model-id",
		Page:      &page,
		PageSize:  &pageSize,
	})
	if err != nil {
		fmt.Printf("ListItems error: %v\n", err)
	} else {
		fmt.Printf("ListItems success: %v\n", itemsResp)
	}
}