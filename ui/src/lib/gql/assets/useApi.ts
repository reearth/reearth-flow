import { useCallback } from "react";

import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import { Asset, CreateAsset, DeleteAsset, UpdateAsset } from "@flow/types";
import type { PaginationOptions } from "@flow/types/paginationOptions";

import {
  CreateAssetInput,
  UpdateAssetInput,
  DeleteAssetInput,
  CreateAssetUploadInput,
} from "../__gen__/graphql";

import { useQueries } from "./useQueries";
// Files larger than 30MB will use direct upload
const MAX_STANDARD_UPLOAD_SIZE_MB = 30;

export const useAsset = () => {
  const {
    useGetAssetsQuery,
    createAssetWithStandardUploadMutation,
    updateAssetMutation,
    deleteAssetMutation,
    createAssetDirectUploadMutation,
  } = useQueries();
  const { toast } = useToast();
  const t = useT();
  const useGetAssets = (
    workspaceId: string,
    keyword?: string,
    paginationOptions?: PaginationOptions,
  ) => {
    const { data, ...rest } = useGetAssetsQuery(
      workspaceId,
      keyword,
      paginationOptions,
    );
    return {
      page: data,
      ...rest,
    };
  };

  const updateAsset = async (input: UpdateAssetInput): Promise<UpdateAsset> => {
    const { mutateAsync, ...rest } = updateAssetMutation;
    try {
      const asset: Asset | undefined = await mutateAsync(input);
      toast({
        title: t("Asset Updated"),
        description: t("Asset has been successfully updated."),
      });
      return { asset, ...rest };
    } catch (_err) {
      toast({
        title: t("Asset Could Not Be Updated"),
        description: t("There was an error when updating the asset."),
        variant: "destructive",
      });
      return { asset: undefined, ...rest };
    }
  };

  const deleteAsset = async (
    assetId: DeleteAssetInput,
  ): Promise<DeleteAsset> => {
    const { mutateAsync, ...rest } = deleteAssetMutation;
    try {
      const data = await mutateAsync(assetId);
      toast({
        title: t("Successful Deletion"),
        description: t(
          "Asset has been successfully deleted from your workspace.",
        ),
      });
      return { assetId: data.assetId, ...rest };
    } catch (_err) {
      toast({
        title: t("Asset Could Not Be Deleted"),
        description: t("There was an error when deleting the asset."),
        variant: "destructive",
      });
      return { assetId: undefined, ...rest };
    }
  };

  const createAssetUploadUrl = useCallback(
    async (input: CreateAssetUploadInput) => {
      const { mutateAsync, ...rest } = createAssetDirectUploadMutation;

      try {
        const assetUpload = await mutateAsync({
          filename: input.filename,
          workspaceId: input.workspaceId,
        });

        return { assetUpload, ...rest };
      } catch (_err) {
        return { assetUpload: undefined, ...rest };
      }
    },
    [createAssetDirectUploadMutation],
  );

  // Create asset with standard upload for files < 30MB
  const createAssetWithStandardUpload = useCallback(
    async (input: CreateAssetInput): Promise<CreateAsset> => {
      const { mutateAsync, ...rest } = createAssetWithStandardUploadMutation;

      try {
        const asset: Asset | undefined = await mutateAsync({
          workspaceId: input.workspaceId,
          file: input.file,
          token: input.token,
        });
        toast({
          title: t("Asset Created"),
          description: t("Asset has been successfully created."),
        });
        return { asset, ...rest };
      } catch (_err) {
        toast({
          title: t("Asset Could Not Be Created"),
          description: t("There was an error when creating the asset."),
          variant: "destructive",
        });
        return { asset: undefined, ...rest };
      }
    },
    [createAssetWithStandardUploadMutation, toast, t],
  );

  // Create asset with direct upload for files > 30MB
  const createAssetWithDirectUpload = useCallback(
    async (input: {
      workspaceId: string;
      file: File;
    }): Promise<CreateAsset> => {
      const { workspaceId, file } = input;
      const { mutateAsync, ...rest } = createAssetWithStandardUploadMutation;
      try {
        const { assetUpload } = await createAssetUploadUrl({
          workspaceId,
          filename: file.name,
        });

        if (!assetUpload?.url || !assetUpload?.token) {
          throw new Error("Failed to get upload URL");
        }

        const uploadResponse = await fetch(assetUpload.url, {
          method: "PUT",
          body: file,
          headers: {
            "Content-Type": assetUpload.contentType || file.type,
            ...(assetUpload.contentEncoding && {
              "Content-Encoding": assetUpload.contentEncoding,
            }),
          },
        });

        if (!uploadResponse.ok) {
          throw new Error(`Upload failed: ${uploadResponse.statusText}`);
        }

        const asset: Asset | undefined = await mutateAsync({
          workspaceId,
          token: assetUpload.token,
        });

        toast({
          title: t("Asset Created"),
          description: t("Asset has been successfully created."),
        });

        return { asset, ...rest };
      } catch (err) {
        console.error("Direct upload failed:", err);
        toast({
          title: t("Asset Could Not Be Created"),
          description: t("There was an error when creating the asset."),
          variant: "destructive",
        });
        return { asset: undefined, ...rest };
      }
    },
    [createAssetUploadUrl, createAssetWithStandardUploadMutation, toast, t],
  );

  // Unified createAsset function
  const createAsset = useCallback(
    async (workspaceId: string, file: File) => {
      const bytesInAMegabyte = 1024 * 1024;
      const maxStandardUploadSize =
        MAX_STANDARD_UPLOAD_SIZE_MB * bytesInAMegabyte;
      if (file.size > maxStandardUploadSize) {
        await createAssetWithDirectUpload({
          workspaceId,
          file,
        });
      } else {
        await createAssetWithStandardUpload({
          workspaceId,
          file,
        });
      }
    },
    [createAssetWithStandardUpload, createAssetWithDirectUpload],
  );

  return {
    useGetAssets,
    createAsset,
    createAssetUploadUrl,
    createAssetWithDirectUpload,
    updateAsset,
    deleteAsset,
  };
};
