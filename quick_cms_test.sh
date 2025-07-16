#!/bin/bash

# 快速 CMS gRPC 测试脚本
# 使用发现的真实 ID 进行测试

PROTO_FILE="server/api/pkg/cms/proto/schema.proto"
ENDPOINT="grpc.cms.dev.reearth.io:443"
TOKEN="fuewiqhriiu38475y42fd"
USER="test-user"
WORKSPACE="01jy5pem6swjmkj7q6zfbgzxk5"
PROJECT="01k06ybhb4km5s8cpe0c93xeda"

AUTH_HEADERS=(-H "authorization: Bearer $TOKEN" -H "user-id: $USER")

echo "测试 CMS gRPC 服务..."
echo "端点: $ENDPOINT"
echo "项目 ID: $PROJECT"
echo "工作空间 ID: $WORKSPACE"
echo ""

# 1. 列出服务
echo "=== 1. 列出服务 ==="
grpcurl -proto "$PROTO_FILE" "${AUTH_HEADERS[@]}" "$ENDPOINT" list

# 2. 获取项目
echo -e "\n=== 2. 获取项目 ==="
grpcurl -proto "$PROTO_FILE" "${AUTH_HEADERS[@]}" \
  -d '{"project_id_or_alias": "'$PROJECT'"}' \
  "$ENDPOINT" reearth.cms.v1.ReEarthCMS/GetProject

# 3. 列出项目
echo -e "\n=== 3. 列出项目 ==="
grpcurl -proto "$PROTO_FILE" "${AUTH_HEADERS[@]}" \
  -d '{"workspace_id": "'$WORKSPACE'", "public_only": true}' \
  "$ENDPOINT" reearth.cms.v1.ReEarthCMS/ListProjects

# 4. 列出模型
echo -e "\n=== 4. 列出模型 ==="
grpcurl -proto "$PROTO_FILE" "${AUTH_HEADERS[@]}" \
  -d '{"project_id": "'$PROJECT'"}' \
  "$ENDPOINT" reearth.cms.v1.ReEarthCMS/ListModels

# 5. 检查别名可用性
echo -e "\n=== 5. 检查别名可用性 ==="
grpcurl -proto "$PROTO_FILE" "${AUTH_HEADERS[@]}" \
  -d '{"alias": "test-alias-available"}' \
  "$ENDPOINT" reearth.cms.v1.ReEarthCMS/CheckAliasAvailability

echo -e "\n测试完成！" 