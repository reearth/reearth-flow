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

export const useAsset = () => {
  const {
    useGetAssetsQuery,
    createAssetMutation,
    updateAssetMutation,
    deleteAssetMutation,
    createAssetUploadMutation,
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

  const createAsset = async (input: CreateAssetInput): Promise<CreateAsset> => {
    const { mutateAsync, ...rest } = createAssetMutation;
    const formData = new FormData();
    formData.append("file", input.file);

    try {
      const asset: Asset | undefined = await mutateAsync({
        workspaceId: input.workspaceId,
        file: formData,
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

  const createAssetUpload = async (input: CreateAssetUploadInput) => {
    const { mutateAsync, ...rest } = createAssetUploadMutation;

    try {
      const assetUpload = await mutateAsync({
        filename: input.filename,
        workspaceId: input.workspaceId,
        // contentLength: input.contentLength,
        // contentEncoding: input.contentEncoding,
        // cursor: input.cursor,
      });

      return { assetUpload, ...rest };
    } catch (_err) {
      return { assetUpload: undefined, ...rest };
    }
  };

  const createAssetWithDirectUpload = async (input: {
    workspaceId: string;
    file: File;
  }): Promise<CreateAsset> => {
    const { workspaceId, file } = input;
    try {
      const { assetUpload } = await createAssetUpload({
        workspaceId,
        filename: file.name,
        // contentLength: file.size,
        // contentEncoding: undefined,
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

      const { mutateAsync } = createAssetMutation;
      const asset: Asset | undefined = await mutateAsync({
        workspaceId,
        token: assetUpload.token,
      });

      toast({
        title: t("Asset Created"),
        description: t("Asset has been successfully created."),
      });

      const { mutateAsync: _, ...rest } = createAssetMutation;
      return { asset, ...rest };
    } catch (err) {
      console.error("Direct upload failed:", err);
      toast({
        title: t("Asset Could Not Be Created"),
        description: t("There was an error when creating the asset."),
        variant: "destructive",
      });
      const { mutateAsync: _, ...rest } = createAssetMutation;
      return { asset: undefined, ...rest };
    }
  };

  return {
    useGetAssets,
    createAsset,
    createAssetUpload,
    createAssetWithDirectUpload,
    updateAsset,
    deleteAsset,
  };
};
