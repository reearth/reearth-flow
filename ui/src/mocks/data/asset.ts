export type MockArchiveExtractionStatus =
  | "skipped"
  | "pending"
  | "in_progress"
  | "done"
  | "failed";

export type MockAsset = {
  id: string;
  workspaceId: string;
  createdAt: string;
  fileName: string;
  size: number;
  contentType: string;
  name: string;
  url: string;
  uuid: string;
  flatFiles: boolean;
  public: boolean;
  // archiveExtractionStatus: MockArchiveExtractionStatus;
};

export const mockAssets: MockAsset[] = [
  {
    id: "asset-1",
    workspaceId: "workspace-1",
    createdAt: "2024-05-12T10:32:45.123Z",
    fileName: "BuildingModel.glb",
    size: 5242880, // 5MB
    contentType: "model/gltf-binary",
    name: "BuildingModel.glb",
    url: "https://mockserver.local/assets/buildingmodel.glb",
    uuid: "uuid-1",
    flatFiles: false,
    public: true,
  },
  {
    id: "asset-2",
    workspaceId: "workspace-1",
    createdAt: "1998-05-14T13:45:22.987Z",
    fileName: "SiteMap.png",
    size: 1048576, // 1MB
    contentType: "image/png",
    name: "SiteMap.png",
    url: "https://mockserver.local/assets/sitemap.png",
    uuid: "uuid-2",
    flatFiles: false,
    public: true,
  },
  {
    id: "asset-3",
    workspaceId: "workspace-1",
    createdAt: "2025-05-12T10:32:45.123Z",
    fileName: "BuildingModel2.glb",
    size: 5242880, // 5MB
    contentType: "model/gltf-binary",
    name: "BuildingModel2.glb",
    url: "https://mockserver.local/assets/buildingmodel2.glb",
    uuid: "uuid-3",
    flatFiles: false,
    public: false,
  },
  {
    id: "asset-4",
    workspaceId: "workspace-1",
    createdAt: "2025-05-14T13:45:22.987Z",
    fileName: "SiteMap2.png",
    size: 1048576, // 1MB
    contentType: "image/png",
    name: "SiteMap2.png",
    url: "https://mockserver.local/assets/sitemap2.png",
    uuid: "uuid-4",
    flatFiles: false,
    public: true,
  },
  {
    id: "asset-5",
    workspaceId: "workspace-1",
    createdAt: "2025-05-12T10:32:45.123Z",
    fileName: "BuildingModel3.glb",
    size: 5242880, // 5MB
    contentType: "model/gltf-binary",
    name: "BuildingModel3.glb",
    url: "https://mockserver.local/assets/buildingmodel3.glb",
    uuid: "uuid-5",
    flatFiles: false,
    public: true,
  },
  {
    id: "asset-6",
    workspaceId: "workspace-1",
    createdAt: "2025-05-14T13:45:22.987Z",
    fileName: "SiteMap3.png",
    size: 1048576, // 1MB
    contentType: "image/png",
    name: "SiteMap3.png",
    url: "https://mockserver.local/assets/sitemap3.png",
    uuid: "uuid-6",
    flatFiles: false,
    public: false,
  },
  {
    id: "asset-7",
    workspaceId: "workspace-1",
    createdAt: "2025-05-12T10:32:45.123Z",
    fileName: "BuildingModel4.glb",
    size: 5242880, // 5MB
    contentType: "model/gltf-binary",
    name: "BuildingModel4.glb",
    url: "https://mockserver.local/assets/buildingmodel4.glb",
    uuid: "uuid-7",
    flatFiles: false,
    public: true,
  },
  {
    id: "asset-8",
    workspaceId: "workspace-1",
    createdAt: "2025-05-14T13:45:22.987Z",
    fileName: "SiteMap4.png",
    size: 1048576, // 1MB
    contentType: "image/png",
    name: "SiteMap4.png",
    url: "https://mockserver.local/assets/sitemap4.png",
    uuid: "uuid-8",
    flatFiles: false,
    public: true,
  },
  {
    id: "asset-9",
    workspaceId: "workspace-1",
    createdAt: "2025-05-12T10:32:45.123Z",
    fileName: "BuildingModel5.glb",
    size: 5242880, // 5MB
    contentType: "model/gltf-binary",
    name: "BuildingModel5.glb",
    url: "https://mockserver.local/assets/buildingmodel5.glb",
    uuid: "uuid-9",
    flatFiles: true,
    public: true,
  },
  {
    id: "asset-10",
    workspaceId: "workspace-1",
    createdAt: "2025-05-14T13:45:22.987Z",
    fileName: "SiteMap5.png",
    size: 1048576, // 1MB
    contentType: "image/png",
    name: "SiteMap5.png",
    url: "https://mockserver.local/assets/sitemap5.png",
    uuid: "uuid-10",
    flatFiles: false,
    public: true,
  },
  {
    id: "asset-11",
    workspaceId: "workspace-1",
    createdAt: "2025-05-12T10:32:45.123Z",
    fileName: "BuildingModel6.glb",
    size: 5242880, // 5MB
    contentType: "model/gltf-binary",
    name: "BuildingModel6.glb",
    url: "https://mockserver.local/assets/buildingmodel6.glb",
    uuid: "uuid-11",
    flatFiles: true,
    public: false,
  },
  {
    id: "asset-12",
    workspaceId: "workspace-1",
    createdAt: "2025-05-14T13:45:22.987Z",
    fileName: "SiteMap6.png",
    size: 1048576, // 1MB
    contentType: "image/png",
    name: "SiteMap6.png",
    url: "https://mockserver.local/assets/sitemap6.png",
    uuid: "uuid-12",
    flatFiles: false,
    public: true,
  },
  {
    id: "asset-13",
    workspaceId: "workspace-2",
    createdAt: "2025-05-18T08:22:11.456Z",
    fileName: "Specs.pdf",
    size: 2097152, // 2MB
    contentType: "application/pdf",
    name: "Specs.pdf",
    url: "https://mockserver.local/assets/specs.pdf",
    uuid: "uuid-13",
    flatFiles: false,
    public: true,
  },
  {
    id: "asset-14",
    workspaceId: "workspace-1",
    createdAt: "2025-05-19T14:22:33.789Z",
    fileName: "CityBoundaries.geojson",
    size: 3145728, // 3MB
    contentType: "application/geo+json",
    name: "CityBoundaries.geojson",
    url: "https://mockserver.local/assets/cityboundaries.geojson",
    uuid: "uuid-14",
    flatFiles: false,
    public: true,
  },
  {
    id: "asset-15",
    workspaceId: "workspace-1",
    createdAt: "2025-05-20T09:17:42.123Z",
    fileName: "TrafficData.json",
    size: 524288, // 512KB
    contentType: "application/json",
    name: "TrafficData.json",
    url: "https://mockserver.local/assets/trafficdata.json",
    uuid: "uuid-15",
    flatFiles: false,
    public: false,
  },
  {
    id: "asset-16",
    workspaceId: "workspace-2",
    createdAt: "2025-05-21T11:33:27.456Z",
    fileName: "BuildingFootprints.geojson",
    size: 2097152, // 2MB
    contentType: "application/geo+json",
    name: "BuildingFootprints.geojson",
    url: "https://mockserver.local/assets/buildingfootprints.geojson",
    uuid: "uuid-16",
    flatFiles: false,
    public: true,
  },
  {
    id: "asset-17",
    workspaceId: "workspace-2",
    createdAt: "2025-05-22T16:05:19.789Z",
    fileName: "ProjectConfig.json",
    size: 102400, // 100KB
    contentType: "application/json",
    name: "ProjectConfig.json",
    url: "https://mockserver.local/assets/projectconfig.json",
    uuid: "uuid-17",
    flatFiles: false,
    public: true,
  },
  {
    id: "asset-18",
    workspaceId: "workspace-1",
    createdAt: "2025-05-22T16:05:19.789Z",
    fileName: "ProjectConfig.json",
    size: 102400, // 100KB
    contentType: "application/json",
    name: "ProjectConfig.json",
    url: "https://mockserver.local/assets/projectconfig.json",
    uuid: "uuid-18",
    flatFiles: false,
    public: true,
  },
  {
    id: "asset-19",
    workspaceId: "workspace-1",
    createdAt: "2025-05-22T16:05:19.789Z",
    fileName: "SearchJson.json",
    size: 102400, // 100KB
    contentType: "application/json",
    name: "SearchJson.json",
    url: "https://mockserver.local/assets/searchjson.json",
    uuid: "uuid-19",
    flatFiles: false,
    public: true,
  },
];
