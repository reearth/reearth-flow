#!/bin/bash

# 配置了正确认证的 GraphQL CMS 测试脚本

# 设置 Auth0 配置环境变量
export REEARTH_FLOW_AUTH_ISS="https://reearth-oss-test.eu.auth0.com/"
export REEARTH_FLOW_AUTH_AUD="k6F1sgFikzVkkcW9Cpz7Ztvwq5cBRXlv"

# 设置 CMS 配置
export REEARTH_FLOW_CMS_ENDPOINT="grpc.cms.dev.reearth.io:443"
export REEARTH_FLOW_CMS_TOKEN="fuewiqhriiu38475y42fd"
export REEARTH_FLOW_CMS_USER_ID="auth0|66d594cc5299fa7d7ade649d"

# 设置跳过权限检查（开发环境）
export REEARTH_FLOW_SKIP_PERMISSION_CHECK="true"

# JWT Token
JWT_TOKEN="eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Im0zZVA3clBDc2YzaGd0V1Uxc2VNWiJ9.eyJuaWNrbmFtZSI6IngueXUiLCJuYW1lIjoieC55dUBldWthcnlhLmlvIiwicGljdHVyZSI6Imh0dHBzOi8vcy5ncmF2YXRhci5jb20vYXZhdGFyL2UyYzhlOTExNWQwYzI3MTE0YTEzMWVjZTZhN2E5NzlkP3M9NDgwJnI9cGcmZD1odHRwcyUzQSUyRiUyRmNkbi5hdXRoMC5jb20lMkZhdmF0YXJzJTJGeC5wbmciLCJ1cGRhdGVkX2F0IjoiMjAyNS0wNy0xNVQwNjo1NTo0NS40NTVaIiwiZW1haWwiOiJ4Lnl1QGV1a2FyeWEuaW8iLCJlbWFpbF92ZXJpZmllZCI6dHJ1ZSwiaXNzIjoiaHR0cHM6Ly9yZWVhcnRoLW9zcy10ZXN0LmV1LmF1dGgwLmNvbS8iLCJhdWQiOiJrNkYxc2dGaWt6VmtrY1c5Q3B6N1p0dndxNWNCUlhsdiIsInN1YiI6ImF1dGgwfDY2ZDU5NGNjNTI5OWZhN2Q3YWRlNjQ5ZCIsImlhdCI6MTc1MjU2MjU0NywiZXhwIjoxNzUyNTk4NTQ3LCJzaWQiOiJ0c0hjZGZjOF9SRE9vMzBBQnpuY296VXg1UjRsU1RQdSIsIm5vbmNlIjoiVEZoZlRVUm9aR2hGUlZwME4wc3paMDVxVVZoWGEzNXdjRUZSYUhGRWJsUlNlRzlpUlRkMFNYWXVXQT09In0.BVPsu11FCyzAIjnXxnnpR-LNVSiJK3nSOmyJuS9L6D4lJ9Q283ViHDA5eJ_cKM7P6vz-Q1XTGrK1EKdJ83W4NyB_FARVd907K-uzrV4WrYyI6dRrxbPkcK6A-ZjbithhEnSNXIuP6BdSHW2s7xfBfelWWwTAYEICe7eBQ7vLcfODgwK0ijSVgCuwZgYSSGlULMn-ctUvvCzzFlnqdoD3tWEPG4aGVzxRHwj_IQu5ebPFx7qObh3GUbsQ_iRH_TtDtVrE1vMs_eq4BqKW5TOMCBtYiX9rO3Ty95pr9qp6ugLp3CFLNk5rFIsa4Dcs6i4tfUwuxx6OrPTFVUygpSqLug"

GRAPHQL_ENDPOINT="http://localhost:8080/api/graphql"
WORKSPACE_ID="01jy5pem6swjmkj7q6zfbgzxk5"
PROJECT_ID="01k06ybhb4km5s8cpe0c93xeda"

echo "========================================="
echo "重新启动服务器以应用认证配置..."
echo "========================================="

# 杀死现有的服务器进程
pkill -f "reearth-flow" || true
sleep 2

# 启动配置了正确认证的服务器
echo "启动带有正确认证配置的服务器..."
cd server/api && go run ./cmd/reearth-flow/ --dev &
SERVER_PID=$!

# 等待服务器启动
echo "等待服务器启动..."
sleep 5

echo "========================================="
echo "1. 测试用户认证"
echo "========================================="
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{"query": "query { me { id name email } }"}' | jq .

echo -e "\n========================================="
echo "2. 测试 CMS 项目列表查询"
echo "========================================="
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{
    "query": "query { cmsProjects(workspaceId: \"'$WORKSPACE_ID'\", publicOnly: true) { id name alias workspaceId visibility createdAt updatedAt } }"
  }' | jq .

echo -e "\n========================================="
echo "3. 测试获取特定 CMS 项目"
echo "========================================="
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{
    "query": "query { cmsProject(projectIdOrAlias: \"test-project\") { id name alias description workspaceId visibility createdAt updatedAt } }"
  }' | jq .

echo -e "\n========================================="
echo "4. 测试 CMS 模型列表查询"
echo "========================================="
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{
    "query": "query { cmsModels(projectId: \"'$PROJECT_ID'\") { id name description key projectId publicApiEp editorUrl createdAt updatedAt } }"
  }' | jq .

echo -e "\n========================================="
echo "5. 测试复杂的 CMS 查询"
echo "========================================="
curl -s -X POST "$GRAPHQL_ENDPOINT" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
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

# 清理
echo "清理服务器进程..."
kill $SERVER_PID 2>/dev/null || true

echo "注意事项："
echo "1. 使用了有效的 Auth0 JWT token"
echo "2. 配置了正确的 issuer 和 audience"
echo "3. 设置了 CMS 端点和认证信息"
echo "4. 启用了权限检查跳过（开发环境）" 