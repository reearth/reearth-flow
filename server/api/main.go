package main

import (
	"context"
	"fmt"
	"log"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/reearthcms"
	"github.com/reearth/reearth-flow/api/pkg/cms"
)

func main() {
	// Configuration
	endpoint := "grpc.cms.dev.reearth.io:443"
	token := "fuewiqhriiu38475y42fd"
	userID := "test-user"
	workspaceID := "01jy5pem6swjmkj7q6zfbgzxk5"
	projectID := "01k06ybhb4km5s8cpe0c93xeda"

	fmt.Println("========================================")
	fmt.Println("Re:Earth Flow CMS Internal Methods Complete Test")
	fmt.Println("========================================")
	fmt.Printf("Endpoint: %s\n", endpoint)
	fmt.Printf("Token: %s...\n", token[:20])
	fmt.Printf("User ID: %s\n", userID)
	fmt.Printf("Workspace ID: %s\n", workspaceID)
	fmt.Printf("Project ID: %s\n", projectID)
	fmt.Println()

	// Create properly configured CMS client with authentication and TLS
	cmsClient, err := reearthcms.NewClient(endpoint, token, userID)
	if err != nil {
		log.Fatalf("Failed to create CMS client: %v", err)
	}

	// Create context
	ctx := context.Background()
	
	// Authentication is now properly configured via metadata
	fmt.Printf("Note: Token %s... and userID %s are configured for authentication\n", token[:10], userID)

	// Test counters
	successCount := 0
	totalTests := 0

	// Test GetProject
	fmt.Println("1. Testing GetProject:")
	fmt.Println("----------------------------------------")
	totalTests++
	project, err := cmsClient.GetProject(ctx, projectID)
	if err != nil {
		fmt.Printf("  ❌ Error: %v\n", err)
	} else {
		fmt.Printf("  ✅ Success: Project Name=%s, ID=%s\n", project.Name, project.ID)
		fmt.Printf("     Details: Alias=%s, Workspace ID=%s\n", project.Alias, project.WorkspaceID)
		if project.Description != nil {
			fmt.Printf("     Description: %s\n", *project.Description)
		}
		fmt.Printf("     Visibility: %s\n", project.Visibility)
		fmt.Printf("     Created At: %s\n", project.CreatedAt)
		fmt.Printf("     Updated At: %s\n", project.UpdatedAt)
		successCount++
	}

	// Test ListProjects
	fmt.Println("\n2. Testing ListProjects:")
	fmt.Println("----------------------------------------")
	totalTests++
	projects, totalCount, err := cmsClient.ListProjects(ctx, cms.ListProjectsInput{
		WorkspaceID: workspaceID,
		PublicOnly:  true,
	})
	if err != nil {
		fmt.Printf("  ❌ Error: %v\n", err)
	} else {
		fmt.Printf("  ✅ Success: Found %d projects (Total: %d)\n", len(projects), totalCount)
		for i, p := range projects {
			fmt.Printf("     %d. %s (ID: %s, Alias: %s)\n", i+1, p.Name, p.ID, p.Alias)
			fmt.Printf("        Workspace ID: %s, Visibility: %s\n", p.WorkspaceID, p.Visibility)
		}
		successCount++
	}

	// Test ListModels
	fmt.Println("\n3. Testing ListModels:")
	fmt.Println("----------------------------------------")
	totalTests++
	models, totalCount, err := cmsClient.ListModels(ctx, cms.ListModelsInput{
		ProjectID: projectID,
	})
	if err != nil {
		fmt.Printf("  ❌ Error: %v\n", err)
	} else {
		fmt.Printf("  ✅ Success: Found %d models (Total: %d)\n", len(models), totalCount)
		for i, m := range models {
			fmt.Printf("     %d. %s (ID: %s, Key: %s)\n", i+1, m.Name, m.ID, m.Key)
			fmt.Printf("        Description: %s\n", m.Description)
			fmt.Printf("        API Endpoint: %s\n", m.PublicAPIEP)
			fmt.Printf("        Editor URL: %s\n", m.EditorURL)
		}
		successCount++
	}

	// Test GetModelGeoJSONExportURL
	fmt.Println("\n4. Testing GetModelGeoJSONExportURL:")
	fmt.Println("----------------------------------------")
	totalTests++
	exportOutput, err := cmsClient.GetModelGeoJSONExportURL(ctx, cms.ExportInput{
		ProjectID: projectID,
		ModelID:   "test-model-id",
	})
	if err != nil {
		fmt.Printf("  ❌ Error: %v\n", err)
	} else {
		fmt.Printf("  ✅ Success: Export URL=%s\n", exportOutput.URL)
		successCount++
	}

	// Test ListItems
	fmt.Println("\n5. Testing ListItems:")
	fmt.Println("----------------------------------------")
	totalTests++
	itemsOutput, err := cmsClient.ListItems(ctx, cms.ListItemsInput{
		ProjectID: projectID,
	})
	if err != nil {
		fmt.Printf("  ❌ Error: %v\n", err)
	} else {
		fmt.Printf("  ✅ Success: Found %d items (Total: %d)\n", len(itemsOutput.Items), itemsOutput.TotalCount)
		for i, item := range itemsOutput.Items {
			fmt.Printf("     %d. Item ID: %s\n", i+1, item.ID)
			if len(item.Fields) > 0 {
				fmt.Printf("        Fields: %d\n", len(item.Fields))
			}
		}
		successCount++
	}

	fmt.Println("\n========================================")
	fmt.Println("Test Summary:")
	fmt.Println("----------------------------------------")
	fmt.Printf("Total Tests: %d\n", totalTests)
	fmt.Printf("Successful Tests: %d\n", successCount)
	fmt.Printf("Failed Tests: %d\n", totalTests-successCount)
	fmt.Printf("Success Rate: %.1f%%\n", float64(successCount)/float64(totalTests)*100)

	fmt.Printf("\nClient Type: Kitex gRPC Client\n")
	fmt.Printf("Endpoint: %s\n", endpoint)
	fmt.Printf("Authentication: Note - should be handled via middleware in production\n")

	fmt.Println("\n========================================")
	fmt.Println("Available gRPC Methods:")
	fmt.Println("----------------------------------------")
	methods := []string{
		"GetProject",
		"ListProjects",
		"ListModels",
		"ListItems",
		"CheckAliasAvailability",
		"GetModelGeoJSONExportURL",
		"CreateProject",
		"UpdateProject",
		"DeleteProject",
	}

	for i, method := range methods {
		fmt.Printf("  %d. %s\n", i+1, method)
	}

	fmt.Println("\n========================================")
	fmt.Println("✅ CMS Internal Methods Test Complete!")
	fmt.Println("========================================")
}