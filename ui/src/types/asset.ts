import type { ApiResponse } from "./api";

export enum AssetSortType {
  Date = "DATE",
  Size = "SIZE",
  Name = "NAME",
}
export type Asset = {
  id: string;
  name: string;
  workspaceId: string;
  createdAt: string;
  contentType: string;
  size: string;
  url: string;
};

export type CreateAsset = {
  asset?: Asset;
} & ApiResponse;

export type RemoveAsset = {
  assetId?: string;
} & ApiResponse;
