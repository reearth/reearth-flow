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
    createdAt: "2024-05-12T10:32:45.123Z",
    size: 5242880, // 5MB
    url: "https://mockserver.local/assets/buildingmodel.glb",
    workspaceId: "workspace-1",
  },
  {
    id: "asset-2",
    name: "SiteMap.png",
    contentType: "image/png",
    createdAt: "1998-05-14T13:45:22.987Z",
    size: 1048576, // 1MB
    url: "https://mockserver.local/assets/sitemap.png",
    workspaceId: "workspace-1",
  },
  {
    id: "asset-3",
    name: "BuildingModel2.glb",
    contentType: "model/gltf-binary",
    createdAt: "2025-05-12T10:32:45.123Z",
    size: 5242880, // 5MB
    url: "https://mockserver.local/assets/buildingmodel2.glb",
    workspaceId: "workspace-1",
  },
  {
    id: "asset-4",
    name: "SiteMap2.png",
    contentType: "image/png",
    createdAt: "2025-05-14T13:45:22.987Z",
    size: 1048576, // 1MB
    url: "https://mockserver.local/assets/sitemap2.png",
    workspaceId: "workspace-1",
  },
  {
    id: "asset-5",
    name: "BuildingModel3.glb",
    contentType: "model/gltf-binary",
    createdAt: "2025-05-12T10:32:45.123Z",
    size: 5242880, // 5MB
    url: "https://mockserver.local/assets/buildingmodel3.glb",
    workspaceId: "workspace-1",
  },
  {
    id: "asset-6",
    name: "SiteMap3.png",
    contentType: "image/png",
    createdAt: "2025-05-14T13:45:22.987Z",
    size: 1048576, // 1MB
    url: "https://mockserver.local/assets/sitemap3.png",
    workspaceId: "workspace-1",
  },
  {
    id: "asset-7",
    name: "BuildingModel4.glb",
    contentType: "model/gltf-binary",
    createdAt: "2025-05-12T10:32:45.123Z",
    size: 5242880, // 5MB
    url: "https://mockserver.local/assets/buildingmodel4.glb",
    workspaceId: "workspace-1",
  },
  {
    id: "asset-8",
    name: "SiteMap4.png",
    contentType: "image/png",
    createdAt: "2025-05-14T13:45:22.987Z",
    size: 1048576, // 1MB
    url: "https://mockserver.local/assets/sitemap4.png",
    workspaceId: "workspace-1",
  },
  {
    id: "asset-9",
    name: "BuildingModel5.glb",
    contentType: "model/gltf-binary",
    createdAt: "2025-05-12T10:32:45.123Z",
    size: 5242880, // 5MB
    url: "https://mockserver.local/assets/buildingmodel5.glb",
    workspaceId: "workspace-1",
  },
  {
    id: "asset-10",
    name: "SiteMap5.png",
    contentType: "image/png",
    createdAt: "2025-05-14T13:45:22.987Z",
    size: 1048576, // 1MB
    url: "https://mockserver.local/assets/sitemap5.png",
    workspaceId: "workspace-1",
  },
  {
    id: "asset-11",
    name: "BuildingModel6.glb",
    contentType: "model/gltf-binary",
    createdAt: "2025-05-12T10:32:45.123Z",
    size: 5242880, // 5MB
    url: "https://mockserver.local/assets/buildingmodel6.glb",
    workspaceId: "workspace-1",
  },
  {
    id: "asset-12",
    name: "SiteMap6.png",
    contentType: "image/png",
    createdAt: "2025-05-14T13:45:22.987Z",
    size: 1048576, // 1MB
    url: "https://mockserver.local/assets/sitemap6.png",
    workspaceId: "workspace-1",
  },
  {
    id: "asset-13",
    name: "Specs.pdf",
    contentType: "application/pdf",
    createdAt: "2025-05-18T08:22:11.456Z",
    size: 2097152, // 2MB
    url: "https://mockserver.local/assets/specs.pdf",
    workspaceId: "workspace-2",
  },
  {
    id: "asset-14",
    name: "CityBoundaries.geojson",
    contentType: "application/geo+json",
    createdAt: "2025-05-19T14:22:33.789Z",
    size: 3145728, // 3MB
    url: "https://mockserver.local/assets/cityboundaries.geojson",
    workspaceId: "workspace-1",
  },
  {
    id: "asset-15",
    name: "TrafficData.json",
    contentType: "application/json",
    createdAt: "2025-05-20T09:17:42.123Z",
    size: 524288, // 512KB
    url: "https://mockserver.local/assets/trafficdata.json",
    workspaceId: "workspace-1",
  },
  {
    id: "asset-16",
    name: "BuildingFootprints.geojson",
    contentType: "application/geo+json",
    createdAt: "2025-05-21T11:33:27.456Z",
    size: 2097152, // 2MB
    url: "https://mockserver.local/assets/buildingfootprints.geojson",
    workspaceId: "workspace-2",
  },
  {
    id: "asset-17",
    name: "ProjectConfig.json",
    contentType: "application/json",
    createdAt: "2025-05-22T16:05:19.789Z",
    size: 102400, // 100KB
    url: "https://mockserver.local/assets/projectconfig.json",
    workspaceId: "workspace-2",
  },
];
