#!/bin/bash

# GraphQL CMS 测试脚本
# 测试 Re:Earth Flow 的 CMS GraphQL 端点

GRAPHQL_ENDPOINT="http://localhost:8080/api/graphql"
WORKSPACE_ID="01jy5pem6swjmkj7q6zfbgzxk5"
PROJECT_ID="01k06ybhb4km5s8cpe0c93xeda"
USER_ID="01jy5pem6swjmkj7q6zfbgzxk6"

# 测试 schema introspection
echo "========================================="
echo "1. 测试 GraphQL Schema Introspection"
echo "========================================="
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d '{"query": "{ __schema { queryType { name } } }"}' | jq .

echo -e "\n========================================="
echo "2. 测试 CMS 项目列表查询"
echo "========================================="
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query { cmsProjects(workspaceId: \"'$WORKSPACE_ID'\", publicOnly: true) { id name alias workspaceId visibility createdAt updatedAt } }"
  }' | jq .

echo -e "\n========================================="
echo "3. 测试获取特定 CMS 项目"
echo "========================================="
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query { cmsProject(projectIdOrAlias: \"test-project\") { id name alias description workspaceId visibility createdAt updatedAt } }"
  }' | jq .

echo -e "\n========================================="
echo "4. 测试 CMS 模型列表查询"
echo "========================================="
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query { cmsModels(projectId: \"'$PROJECT_ID'\") { id name description key projectId publicApiEp editorUrl createdAt updatedAt } }"
  }' | jq .

echo -e "\n========================================="
echo "5. 测试 CMS 项目和模型的完整查询"
echo "========================================="
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query { cmsProject(projectIdOrAlias: \"'$PROJECT_ID'\") { id name alias description workspaceId visibility createdAt updatedAt } cmsModels(projectId: \"'$PROJECT_ID'\") { id name description key } }"
  }' | jq .

echo -e "\n========================================="
echo "6. 测试 CMS 导出 URL"
echo "========================================="
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query { cmsModelExportUrl(projectId: \"'$PROJECT_ID'\", modelId: \"test-model-id\") }"
  }' | jq .

echo -e "\n========================================="
echo "7. 测试认证方式"
echo "========================================="
echo "尝试使用 Bearer token..."
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -H "authorization: Bearer test" \
  -d '{
    "query": "query { cmsProjects(workspaceId: \"'$WORKSPACE_ID'\", publicOnly: true) { id name } }"
  }' | jq .

echo -e "\n尝试使用调试用户头..."
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -H "authorization: Bearer test" \
  -H "X-Reearth-Debug-User: $USER_ID" \
  -d '{
    "query": "query { cmsProjects(workspaceId: \"'$WORKSPACE_ID'\", publicOnly: true) { id name } }"
  }' | jq .

echo -e "\n========================================="
echo "8. 测试复杂查询"
echo "========================================="
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "query GetCMSData($workspaceId: ID!, $projectId: ID!) { cmsProjects(workspaceId: $workspaceId, publicOnly: true) { id name alias workspaceId visibility createdAt updatedAt } cmsModels(projectId: $projectId) { id name description key schema { schemaId fields { fieldId name type key description } } } }",
    "variables": {
      "workspaceId": "'$WORKSPACE_ID'",
      "projectId": "'$PROJECT_ID'"
    }
  }' | jq .

echo -e "\n========================================="
echo "测试完成！"
echo "========================================="
echo "注意事项："
echo "1. 如果看到认证错误，可能需要配置正确的 JWT token"
echo "2. 如果看到 'CMS gateway not configured' 错误，需要配置 CMS 环境变量"
echo "3. 如果看到 'operator not found' 错误，需要提供认证信息"
echo "4. 成功的查询应该返回 JSON 数据而不是错误" 