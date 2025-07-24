package main

import (
	"context"
	"fmt"

	"github.com/reearth/reearth-flow/api/internal/infrastructure/cms"
	"google.golang.org/grpc/metadata"
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

	client, err := cms.NewGRPCClient(endpoint, token, true)
	if err != nil {
		fmt.Printf("  ❌ Error: %v\n", err)
		return
	}

	ctx := context.Background()
	md := metadata.New(map[string]string{
		"authorization": fmt.Sprintf("Bearer %s", token),
	})
	ctx = metadata.NewOutgoingContext(ctx, md)


	resp, err := client.GetProject(ctx, "test-project")
	if err != nil {
		fmt.Printf("  ❌ Error: %v\n", err)
	} else {
		fmt.Printf("  ✅ Success: Project Name=%s, ID=%s\n", resp.Name, resp.ID)
		fmt.Printf("     Details: Alias=%s, Workspace ID=%s\n", resp.Alias, resp.WorkspaceID)
		if resp.Description != nil {
			fmt.Printf("     Description: %s\n", *resp.Description)
		}
		fmt.Printf("     Updated At: %s\n", resp.UpdatedAt)
	}

	// listResp, err := client.ListProjects(ctx, &proto.ListProjectsRequest{
	// 	WorkspaceId: workspaceID,
	// 	PublicOnly:  true,
	// })
	// if err != nil {
	// 	fmt.Printf("  ❌ Error: %v\n", err)
	// } else {
	// 	fmt.Printf("  ✅ Success: Found %d projects (Total: %d)\n", len(listResp.Projects), listResp.TotalCount)
	// 	for i, p := range listResp.Projects {
	// 		fmt.Printf("     %d. %s (ID: %s, Alias: %s)\n", i+1, p.Name, p.Id, p.Alias)
	// 		fmt.Printf("        Workspace ID: %s, Visibility: %s\n", p.WorkspaceId, p.Visibility)
	// 	}
	// 	successCount++
	// }

	// // Test ListModels
	// fmt.Println("\n3. Testing ListModels:")
	// fmt.Println("----------------------------------------")
	// totalTests++
	// modelsResp, err := client.ListModels(ctx, &proto.ListModelsRequest{
	// 	ProjectId: projectID,
	// })
	// if err != nil {
	// 	fmt.Printf("  ❌ Error: %v\n", err)
	// } else {
	// 	fmt.Printf("  ✅ Success: Found %d models (Total: %d)\n", len(modelsResp.Models), modelsResp.TotalCount)
	// 	for i, m := range modelsResp.Models {
	// 		fmt.Printf("     %d. %s (ID: %s, Key: %s)\n", i+1, m.Name, m.Id, m.Key)
	// 		fmt.Printf("        Description: %s\n", m.Description)
	// 		fmt.Printf("        API Endpoint: %s\n", m.PublicApiEp)
	// 		fmt.Printf("        Editor URL: %s\n", m.EditorUrl)
	// 	}
	// 	successCount++
	// }

	// // Test CheckAliasAvailability - New alias
	// fmt.Println("\n4. Testing CheckAliasAvailability (New Alias):")
	// fmt.Println("----------------------------------------")
	// totalTests++
	// aliasResp, err := client.CheckAliasAvailability(ctx, &proto.AliasAvailabilityRequest{
	// 	Alias: "test-alias-available",
	// })
	// if err != nil {
	// 	fmt.Printf("  ❌ Error: %v\n", err)
	// } else {
	// 	fmt.Printf("  ✅ Success: Alias 'test-alias-available' Available=%t\n", aliasResp.Available)
	// 	successCount++
	// }

	// // Test CheckAliasAvailability - Existing alias
	// fmt.Println("\n5. Testing CheckAliasAvailability (Existing Alias):")
	// fmt.Println("----------------------------------------")
	// totalTests++
	// existingAliasResp, err := client.CheckAliasAvailability(ctx, &proto.AliasAvailabilityRequest{
	// 	Alias: "test-project",
	// })
	// if err != nil {
	// 	fmt.Printf("  ❌ Error: %v\n", err)
	// } else {
	// 	fmt.Printf("  ✅ Success: Alias 'test-project' Available=%t\n", existingAliasResp.Available)
	// 	successCount++
	// }

	// // Test GetModelGeoJSONExportURL
	// fmt.Println("\n6. Testing GetModelGeoJSONExportURL:")
	// fmt.Println("----------------------------------------")
	// totalTests++
	// exportResp, err := client.GetModelGeoJSONExportURL(ctx, &proto.ExportRequest{
	// 	ProjectId: projectID,
	// 	ModelId:   "test-model-id",
	// })
	// if err != nil {
	// 	fmt.Printf("  ❌ Error: %v\n", err)
	// } else {
	// 	fmt.Printf("  ✅ Success: Export URL=%s\n", exportResp.Url)
	// 	successCount++
	// }

	// // Test ListItems
	// fmt.Println("\n7. Testing ListItems:")
	// fmt.Println("----------------------------------------")
	// totalTests++
	// itemsResp, err := client.ListItems(ctx, &proto.ListItemsRequest{
	// 	ProjectId: projectID,
	// 	ModelId:   "test-model-id",
	// 	Page:      &[]int32{1}[0],
	// 	PageSize:  &[]int32{10}[0],
	// })
	// if err != nil {
	// 	fmt.Printf("  ❌ Error: %v\n", err)
	// } else {
	// 	fmt.Printf("  ✅ Success: Found %d items (Total: %d)\n", len(itemsResp.Items), itemsResp.TotalCount)
	// 	for i, item := range itemsResp.Items {
	// 		fmt.Printf("     %d. Item ID: %s\n", i+1, item.Id)
	// 		fmt.Printf("        Field Count: %d\n", len(item.Fields))
	// 		fmt.Printf("        Created At: %s\n", item.CreatedAt.AsTime())
	// 		fmt.Printf("        Updated At: %s\n", item.UpdatedAt.AsTime())
	// 	}
	// 	successCount++
	// }

	// fmt.Println("\n========================================")
	// fmt.Println("Test Summary:")
	// fmt.Println("----------------------------------------")
	// fmt.Printf("Total Tests: %d\n", totalTests)
	// fmt.Printf("Successful Tests: %d\n", successCount)
	// fmt.Printf("Failed Tests: %d\n", totalTests-successCount)
	// fmt.Printf("Success Rate: %.1f%%\n", float64(successCount)/float64(totalTests)*100)

	// fmt.Printf("\nConnection State: %s\n", conn.GetState())
	// fmt.Printf("Using TLS: Yes\n")
	// fmt.Printf("Authentication: Bearer token + User ID\n")

	// fmt.Println("\n========================================")
	// fmt.Println("Available gRPC Methods:")
	// fmt.Println("----------------------------------------")
	// methods := []string{
	// 	"GetProject",
	// 	"ListProjects",
	// 	"ListModels",
	// 	"ListItems",
	// 	"CheckAliasAvailability",
	// 	"GetModelGeoJSONExportURL",
	// 	"CreateProject",
	// 	"UpdateProject",
	// 	"DeleteProject",
	// }

	// for i, method := range methods {
	// 	fmt.Printf("  %d. %s\n", i+1, method)
	// }

	// fmt.Println("\n========================================")
	// fmt.Println("✅ CMS Internal Methods Test Complete!")
	// fmt.Println("========================================")
}
