import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import { Asset, CreateAsset, RemoveAsset } from "@flow/types";
import type { PaginationOptions } from "@flow/types/paginationOptions";

import { CreateAssetInput, RemoveAssetInput } from "../__gen__/graphql";

import { useQueries } from "./useQueries";

export const useAsset = () => {
  const { useGetAssetsQuery, createAssetMutation, removeAssetMutation } =
    useQueries();
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
    try {
      const asset: Asset | undefined = await mutateAsync(input);
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

  const removeAsset = async (
    assetId: RemoveAssetInput,
  ): Promise<RemoveAsset> => {
    const { mutateAsync, ...rest } = removeAssetMutation;
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

  return {
    useGetAssets,
    createAsset,
    removeAsset,
  };
};
