import type { ApiResponse } from "./api";

export enum AssetOrderBy {
  CreatedAt = "createdAt",
  Name = "name",
  Size = "size",
}

export type ArchiveExtractionStatus =
  | "skipped"
  | "pending"
  | "in_progress"
  | "done"
  | "failed";

export type Asset = {
  id: string;
  workspaceId: string;
  createdAt: string;
  fileName: string;
  size: string;
  contentType: string;
  name: string;
  url: string;
  uuid: string;
  flatFiles: boolean;
  public: boolean;
  archiveExtractionStatus: ArchiveExtractionStatus;
};

export type CreateAsset = {
  asset?: Asset;
} & ApiResponse;

export type UpdateAsset = {
  asset?: Asset;
} & ApiResponse;

export type DeleteAsset = {
  assetId?: string;
} & ApiResponse;
