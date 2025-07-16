package main

import (
	"context"
	"crypto/tls"
	"fmt"
	"log"

	"github.com/reearth/reearth-flow/api/pkg/cms/proto"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/metadata"
)

func main() {
	// 配置
	endpoint := "grpc.cms.dev.reearth.io:443"
	token := "fuewiqhriiu38475y42fd"
	userID := "test-user"
	workspaceID := "01jy5pem6swjmkj7q6zfbgzxk5"
	projectID := "01k06ybhb4km5s8cpe0c93xeda"

	fmt.Println("========================================")
	fmt.Println("直接测试 CMS gRPC 客户端 (TLS)")
	fmt.Println("========================================")
	fmt.Printf("端点: %s\n", endpoint)
	fmt.Printf("用户ID: %s\n", userID)
	fmt.Printf("工作空间ID: %s\n", workspaceID)
	fmt.Printf("项目ID: %s\n", projectID)
	fmt.Println()

	// 建立 TLS gRPC 连接
	config := &tls.Config{
		ServerName: "grpc.cms.dev.reearth.io",
	}
	creds := credentials.NewTLS(config)

	conn, err := grpc.NewClient(endpoint, grpc.WithTransportCredentials(creds))
	if err != nil {
		log.Fatalf("Failed to connect: %v", err)
	}
	defer conn.Close()

	client := proto.NewReEarthCMSClient(conn)

	// 创建带认证的 context
	ctx := context.Background()
	md := metadata.New(map[string]string{
		"authorization": fmt.Sprintf("Bearer %s", token),
		"user-id":       userID,
	})
	ctx = metadata.NewOutgoingContext(ctx, md)

	// 测试 GetProject
	fmt.Println("1. 测试 GetProject:")
	fmt.Println("----------------------------------------")
	resp, err := client.GetProject(ctx, &proto.ProjectRequest{
		ProjectIdOrAlias: "test-project",
	})
	if err != nil {
		fmt.Printf("  错误: %v\n", err)
	} else {
		fmt.Printf("  成功: 项目名称=%s, ID=%s\n", resp.Project.Name, resp.Project.Id)
		fmt.Printf("  详细信息: 别名=%s, 工作空间ID=%s\n", resp.Project.Alias, resp.Project.WorkspaceId)
	}

	// 测试 ListProjects
	fmt.Println("\n2. 测试 ListProjects:")
	fmt.Println("----------------------------------------")
	listResp, err := client.ListProjects(ctx, &proto.ListProjectsRequest{
		WorkspaceId: workspaceID,
		PublicOnly:  true,
	})
	if err != nil {
		fmt.Printf("  错误: %v\n", err)
	} else {
		fmt.Printf("  成功: 找到 %d 个项目 (总计: %d)\n", len(listResp.Projects), listResp.TotalCount)
		for i, p := range listResp.Projects {
			fmt.Printf("    %d. %s (ID: %s, 别名: %s)\n", i+1, p.Name, p.Id, p.Alias)
		}
	}

	// 测试 ListModels
	fmt.Println("\n3. 测试 ListModels:")
	fmt.Println("----------------------------------------")
	modelsResp, err := client.ListModels(ctx, &proto.ListModelsRequest{
		ProjectId: projectID,
	})
	if err != nil {
		fmt.Printf("  错误: %v\n", err)
	} else {
		fmt.Printf("  成功: 找到 %d 个模型 (总计: %d)\n", len(modelsResp.Models), modelsResp.TotalCount)
		for i, m := range modelsResp.Models {
			fmt.Printf("    %d. %s (ID: %s, Key: %s)\n", i+1, m.Name, m.Id, m.Key)
		}
	}

	// 测试 CheckAliasAvailability
	fmt.Println("\n4. 测试 CheckAliasAvailability:")
	fmt.Println("----------------------------------------")
	aliasResp, err := client.CheckAliasAvailability(ctx, &proto.AliasAvailabilityRequest{
		Alias: "test-alias-available",
	})
	if err != nil {
		fmt.Printf("  错误: %v\n", err)
	} else {
		fmt.Printf("  成功: 别名 'test-alias-available' 可用性=%t\n", aliasResp.Available)
	}

	// 测试已知存在的别名
	fmt.Println("\n5. 测试已知别名:")
	fmt.Println("----------------------------------------")
	existingAliasResp, err := client.CheckAliasAvailability(ctx, &proto.AliasAvailabilityRequest{
		Alias: "test-project",
	})
	if err != nil {
		fmt.Printf("  错误: %v\n", err)
	} else {
		fmt.Printf("  成功: 别名 'test-project' 可用性=%t\n", existingAliasResp.Available)
	}

	fmt.Println("\n========================================")
	fmt.Println("6. 连接和认证测试总结:")
	fmt.Println("----------------------------------------")
	fmt.Printf("连接状态: %s\n", conn.GetState())
	fmt.Printf("使用 TLS: 是\n")
	fmt.Printf("认证方式: Bearer token + User ID\n")

	fmt.Println("\n========================================")
	fmt.Println("测试完成！")
	fmt.Println("========================================")
}
