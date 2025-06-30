export type MockAsset = {
  id: string;
  name: string;
  contentType: string;
  createdAt: string;
  size: number;
  url: string;
  workspaceId: string;
};

export const mockAssets: MockAsset[] = [
  {
    id: "asset-1",
    name: "BuildingModel.glb",
    contentType: "model/gltf-binary",
    createdAt: "2025-05-12T10:32:45.123Z",
    size: 5242880, // 5MB
    url: "https://mockserver.local/assets/buildingmodel.glb",
    workspaceId: "workspace-1",
  },
  {
    id: "asset-2",
    name: "SiteMap.png",
    contentType: "image/png",
    createdAt: "2025-05-14T13:45:22.987Z",
    size: 1048576, // 1MB
    url: "https://mockserver.local/assets/sitemap.png",
    workspaceId: "workspace-1",
  },
  {
    id: "asset-3",
    name: "Specs.pdf",
    contentType: "application/pdf",
    createdAt: "2025-05-18T08:22:11.456Z",
    size: 2097152, // 2MB
    url: "https://mockserver.local/assets/specs.pdf",
    workspaceId: "workspace-2",
  },
];
