import { useCallback, useState } from "react";

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

import {
  putFileWithProgress,
  type UploadProgress,
  type UploadResult,
} from "./putFileWithProgress";
import { useQueries } from "./useQueries";
// Files larger than 30MB will use direct upload
const MAX_STANDARD_UPLOAD_SIZE_MB = 30;

// A direct upload PUTs the whole file to storage in one request, which on a slow
// connection is legitimately long-running. Give up only once the transfer stops
// making progress, so a slow-but-moving upload is never cut off.
const UPLOAD_STALL_TIMEOUT_MS = 2 * 60 * 1000;

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

  const [uploadProgress, setUploadProgress] = useState<
    UploadProgress | undefined
  >(undefined);
  const useGetAssets = (
    workspaceId?: string,
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

      let stage: "sign" | "upload" | "register" = "sign";
      setUploadProgress({ loaded: 0, total: file.size, percent: 0 });
      try {
        const { assetUpload } = await createAssetUploadUrl({
          workspaceId,
          filename: file.name,
        });

        if (!assetUpload?.url || !assetUpload?.token) {
          throw new Error("Failed to get upload URL");
        }

        stage = "upload";
        const contentType = assetUpload.contentType || file.type;

        let uploadResponse: UploadResult;
        try {
          uploadResponse = await putFileWithProgress({
            url: assetUpload.url,
            file,
            headers: {
              "Content-Type": contentType,
              "Content-Encoding": assetUpload.contentEncoding,
            },
            stallTimeoutMs: UPLOAD_STALL_TIMEOUT_MS,
            onProgress: setUploadProgress,
          });
        } catch (networkErr) {
          throw new Error(
            `PUT to storage never completed: ${(networkErr as Error).message}. No HTTP response was received.`,
          );
        }

        if (uploadResponse.status < 200 || uploadResponse.status >= 300) {
          throw new Error(
            `PUT to storage returned ${uploadResponse.status} ${uploadResponse.statusText}. ${uploadResponse.responseText.slice(0, 500)}`,
          );
        }

        stage = "register";
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
        const stageLabel = {
          sign: "requesting the upload URL",
          upload: "uploading the file to storage",
          register: "registering the uploaded asset",
        }[stage];
        toast({
          title: t("Asset Could Not Be Created"),
          description: t("Failed while {{stage}}: {{reason}}", {
            stage: stageLabel,
            reason: (err as Error).message,
          }),
          variant: "destructive",
        });
        return { asset: undefined, ...rest };
      } finally {
        setUploadProgress(undefined);
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
        return await createAssetWithDirectUpload({
          workspaceId,
          file,
        });
      } else {
        return await createAssetWithStandardUpload({
          workspaceId,
          file,
        });
      }
    },
    [createAssetWithStandardUpload, createAssetWithDirectUpload],
  );

  const isCreatingAsset =
    createAssetWithStandardUploadMutation.isPending ||
    createAssetDirectUploadMutation.isPending ||
    uploadProgress !== undefined;
  return {
    useGetAssets,
    createAsset,
    createAssetUploadUrl,
    createAssetWithDirectUpload,
    isCreatingAsset,
    uploadProgress,
    updateAsset,
    deleteAsset,
  };
};
