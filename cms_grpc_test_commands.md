# CMS gRPC 测试命令集

## 测试配置
```bash
GRPC_ENDPOINT="grpc.cms.dev.reearth.io:443"
GRPC_TOKEN="fuewiqhriiu38475y42fd"
USER_ID="test-user"
WORKSPACE_ID="01jy5pem6swjmkj7q6zfbgzxk5"
PROJECT_ID="01k06ybhb4km5s8cpe0c93xeda"
```

## 基本测试命令

### 1. 列出服务
```bash
grpcurl -proto server/api/pkg/cms/proto/schema.proto \
  -H "authorization: Bearer fuewiqhriiu38475y42fd" \
  -H "user-id: test-user" \
  grpc.cms.dev.reearth.io:443 list
```

### 2. 列出 ReEarthCMS 方法
```bash
grpcurl -proto server/api/pkg/cms/proto/schema.proto \
  -H "authorization: Bearer fuewiqhriiu38475y42fd" \
  -H "user-id: test-user" \
  grpc.cms.dev.reearth.io:443 list reearth.cms.v1.ReEarthCMS
```

### 3. 检查别名可用性
```bash
grpcurl -proto server/api/pkg/cms/proto/schema.proto \
  -H "authorization: Bearer fuewiqhriiu38475y42fd" \
  -H "user-id: test-user" \
  -d '{"alias": "test-alias"}' \
  grpc.cms.dev.reearth.io:443 reearth.cms.v1.ReEarthCMS/CheckAliasAvailability
```

### 4. 获取项目
```bash
grpcurl -proto server/api/pkg/cms/proto/schema.proto \
  -H "authorization: Bearer fuewiqhriiu38475y42fd" \
  -H "user-id: test-user" \
  -d '{"project_id_or_alias": "test-project"}' \
  grpc.cms.dev.reearth.io:443 reearth.cms.v1.ReEarthCMS/GetProject
```

### 5. 列出项目
```bash
grpcurl -proto server/api/pkg/cms/proto/schema.proto \
  -H "authorization: Bearer fuewiqhriiu38475y42fd" \
  -H "user-id: test-user" \
  -d '{"workspace_id": "01jy5pem6swjmkj7q6zfbgzxk5", "public_only": true}' \
  grpc.cms.dev.reearth.io:443 reearth.cms.v1.ReEarthCMS/ListProjects
```

### 6. 列出模型
```bash
grpcurl -proto server/api/pkg/cms/proto/schema.proto \
  -H "authorization: Bearer fuewiqhriiu38475y42fd" \
  -H "user-id: test-user" \
  -d '{"project_id": "01k06ybhb4km5s8cpe0c93xeda"}' \
  grpc.cms.dev.reearth.io:443 reearth.cms.v1.ReEarthCMS/ListModels
```

### 7. 创建项目 (可能需要更高权限)
```bash
grpcurl -proto server/api/pkg/cms/proto/schema.proto \
  -H "authorization: Bearer fuewiqhriiu38475y42fd" \
  -H "user-id: test-user" \
  -d '{
    "workspace_id": "01jy5pem6swjmkj7q6zfbgzxk5",
    "name": "Test Project from grpcurl",
    "description": "This is a test project created via grpcurl",
    "alias": "test-project-grpcurl",
    "visibility": "PUBLIC"
  }' \
  grpc.cms.dev.reearth.io:443 reearth.cms.v1.ReEarthCMS/CreateProject
```

## 测试结果

### 成功的测试：
- ✅ 列出服务
- ✅ 列出方法
- ✅ 检查别名可用性 (返回空响应)
- ✅ 获取项目 (返回项目详情)
- ✅ 列出项目 (返回项目列表)
- ✅ 列出模型 (项目无模型时返回空)

### 失败的测试：
- ❌ 创建项目 (错误: "invalid operator")

## 发现的信息

### 可用的项目：
```json
{
  "id": "01k06ybhb4km5s8cpe0c93xeda",
  "name": "test-project",
  "alias": "test-project",
  "description": "",
  "license": "",
  "readme": "",
  "workspaceId": "01jy5pem6swjmkj7q6zfbgzxk5",
  "createdAt": "2025-07-15T11:43:38.852Z",
  "updatedAt": "2025-07-15T11:43:38.852Z"
}
```

### 可用的方法：
- reearth.cms.v1.ReEarthCMS.CheckAliasAvailability
- reearth.cms.v1.ReEarthCMS.CreateProject
- reearth.cms.v1.ReEarthCMS.DeleteProject
- reearth.cms.v1.ReEarthCMS.GetModelGeoJSONExportURL
- reearth.cms.v1.ReEarthCMS.GetProject
- reearth.cms.v1.ReEarthCMS.ListItems
- reearth.cms.v1.ReEarthCMS.ListModels
- reearth.cms.v1.ReEarthCMS.ListProjects
- reearth.cms.v1.ReEarthCMS.UpdateProject

## 注意事项

1. 服务器不支持 gRPC 反射，需要使用 proto 文件
2. 某些操作可能需要特定的用户权限
3. 创建操作可能需要管理员权限
4. 空响应可能表示成功但无数据返回 