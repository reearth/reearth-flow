import type { ApiResponse } from "./api";

export type AssetSortType = "DATE" | "SIZE" | "NAME";
export type Asset = {
  id: string;
  name: string;
  workspaceId: string;
  createdAt: string;
  contentType: string;
  size: number;
  url: string;
};

export type CreateAsset = {
  asset?: Asset;
} & ApiResponse;

export type RemoveAsset = {
  assetId?: string;
} & ApiResponse;
