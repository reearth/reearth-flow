#!/bin/bash

# CMS gRPC 测试脚本
# 使用 grpcurl 测试 Re:Earth Flow 的 CMS 服务

# 配置变量
GRPC_ENDPOINT="grpc.cms.dev.reearth.io:443"
GRPC_TOKEN="fuewiqhriiu38475y42fd"
USER_ID="test-user-id"  # 如果没有具体的用户ID，请替换为实际值

# 检查 grpcurl 是否安装
if ! command -v grpcurl &> /dev/null; then
    echo "grpcurl 未安装，请先安装：brew install grpcurl"
    exit 1
fi

# 公共 headers
HEADERS=(
    -H "authorization: Bearer $GRPC_TOKEN"
    -H "user-id: $USER_ID"
)

# 1. 测试服务反射和列出可用服务
echo "========================================="
echo "1. 列出可用的 gRPC 服务"
echo "========================================="
grpcurl "${HEADERS[@]}" "$GRPC_ENDPOINT" list

echo ""
echo "========================================="
echo "2. 列出 ReEarthCMS 服务的方法"
echo "========================================="
grpcurl "${HEADERS[@]}" "$GRPC_ENDPOINT" list reearth.cms.v1.ReEarthCMS

echo ""
echo "========================================="
echo "3. 获取 ReEarthCMS 服务的描述"
echo "========================================="
grpcurl "${HEADERS[@]}" "$GRPC_ENDPOINT" describe reearth.cms.v1.ReEarthCMS

echo ""
echo "========================================="
echo "4. 获取 Project 消息类型定义"
echo "========================================="
grpcurl "${HEADERS[@]}" "$GRPC_ENDPOINT" describe reearth.cms.v1.Project

echo ""
echo "========================================="
echo "5. 测试 CheckAliasAvailability 方法"
echo "========================================="
grpcurl "${HEADERS[@]}" \
    -d '{"alias": "test-alias"}' \
    "$GRPC_ENDPOINT" \
    reearth.cms.v1.ReEarthCMS/CheckAliasAvailability

echo ""
echo "========================================="
echo "6. 测试 ListProjects 方法"
echo "========================================="
# 需要替换为实际的 workspace_id
WORKSPACE_ID="test-workspace-id"
grpcurl "${HEADERS[@]}" \
    -d '{"workspace_id": "'$WORKSPACE_ID'", "public_only": true}' \
    "$GRPC_ENDPOINT" \
    reearth.cms.v1.ReEarthCMS/ListProjects

echo ""
echo "========================================="
echo "7. 测试 GetProject 方法"
echo "========================================="
# 需要替换为实际的 project_id 或 alias
PROJECT_ID="test-project-id"
grpcurl "${HEADERS[@]}" \
    -d '{"project_id_or_alias": "'$PROJECT_ID'"}' \
    "$GRPC_ENDPOINT" \
    reearth.cms.v1.ReEarthCMS/GetProject

echo ""
echo "========================================="
echo "8. 测试 ListModels 方法"
echo "========================================="
grpcurl "${HEADERS[@]}" \
    -d '{"project_id": "'$PROJECT_ID'"}' \
    "$GRPC_ENDPOINT" \
    reearth.cms.v1.ReEarthCMS/ListModels

echo ""
echo "========================================="
echo "9. 测试 ListItems 方法"
echo "========================================="
MODEL_ID="test-model-id"
grpcurl "${HEADERS[@]}" \
    -d '{"project_id": "'$PROJECT_ID'", "model_id": "'$MODEL_ID'", "page": 1, "page_size": 10}' \
    "$GRPC_ENDPOINT" \
    reearth.cms.v1.ReEarthCMS/ListItems

echo ""
echo "========================================="
echo "10. 测试 GetModelGeoJSONExportURL 方法"
echo "========================================="
grpcurl "${HEADERS[@]}" \
    -d '{"project_id": "'$PROJECT_ID'", "model_id": "'$MODEL_ID'"}' \
    "$GRPC_ENDPOINT" \
    reearth.cms.v1.ReEarthCMS/GetModelGeoJSONExportURL

echo ""
echo "========================================="
echo "11. 测试 CreateProject 方法"
echo "========================================="
grpcurl "${HEADERS[@]}" \
    -d '{
        "workspace_id": "'$WORKSPACE_ID'",
        "name": "Test Project",
        "description": "Test project created via grpcurl",
        "alias": "test-project-'$(date +%s)'",
        "visibility": "PUBLIC"
    }' \
    "$GRPC_ENDPOINT" \
    reearth.cms.v1.ReEarthCMS/CreateProject

echo ""
echo "========================================="
echo "测试完成！"
echo "========================================="
echo "注意事项："
echo "1. 请替换脚本中的 USER_ID 为实际的用户 ID"
echo "2. 请替换 WORKSPACE_ID 为实际的工作空间 ID"
echo "3. 请替换 PROJECT_ID 为实际的项目 ID"
echo "4. 请替换 MODEL_ID 为实际的模型 ID"
echo "5. 如果服务器不支持反射，您可能需要提供 .proto 文件"
echo ""
echo "Proto 文件位置: server/api/pkg/cms/proto/schema.proto" 