import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { Asset } from "@flow/types";
import {
  OrderDirection,
  type PaginationOptions,
} from "@flow/types/paginationOptions";
import { isDefined } from "@flow/utils";

import {
  CreateAssetInput,
  UpdateAssetInput,
  DeleteAssetInput,
  CreateAssetUploadInput,
} from "../__gen__/graphql";
import { toAsset } from "../convert";
import { useGraphQLContext } from "../provider";

export enum AssetQueryKeys {
  GetAssets = "getAssets",
}

export const ASSET_FETCH_RATE = 30;

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const useGetAssetsQuery = (
    workspaceId: string,
    keyword?: string,
    paginationOptions?: PaginationOptions,
  ) =>
    useQuery({
      queryKey: [AssetQueryKeys.GetAssets, workspaceId],
      queryFn: async () => {
        const data = await graphQLContext?.GetAssets({
          workspaceId: workspaceId ?? "",
          keyword,
          pagination: {
            page: paginationOptions?.page ?? 1,
            pageSize: ASSET_FETCH_RATE,
            orderDir: paginationOptions?.orderDir ?? OrderDirection.Desc,
            orderBy: paginationOptions?.orderBy ?? "createdAt",
          },
        });
        if (!data) return;
        const {
          assets: {
            nodes,
            pageInfo: { totalCount, currentPage, totalPages },
          },
        } = data;

        const assets: Asset[] = nodes
          .filter(isDefined)
          .map((asset) => toAsset(asset));
        return { assets, totalCount, currentPage, totalPages };
      },
      enabled: !!workspaceId,
    });

  const createAssetWithStandardUploadMutation = useMutation({
    mutationFn: async ({
      file,
      name,
      token,
      workspaceId,
    }: CreateAssetInput) => {
      const data = await graphQLContext?.CreateAsset({
        input: {
          file,
          name,
          token,
          workspaceId,
        },
      });

      if (data?.createAsset?.asset) {
        return toAsset(data.createAsset.asset);
      }
    },
    onSuccess: (variables) => {
      queryClient.invalidateQueries({
        queryKey: [AssetQueryKeys.GetAssets, variables?.workspaceId],
      });
    },
  });

  const updateAssetMutation = useMutation({
    mutationFn: async (input: UpdateAssetInput) => {
      const data = await graphQLContext?.UpdateAsset({
        input,
      });

      if (data?.updateAsset?.asset) {
        return toAsset(data.updateAsset.asset);
      }
    },
    onSuccess: (variables) => {
      queryClient.invalidateQueries({
        queryKey: [AssetQueryKeys.GetAssets, variables?.workspaceId],
      });
    },
  });

  const deleteAssetMutation = useMutation({
    mutationFn: async (input: DeleteAssetInput) => {
      const data = await graphQLContext?.DeleteAsset({
        input,
      });

      return {
        assetId: data?.deleteAsset?.assetId,
      };
    },

    onSuccess: () => {
      queryClient.invalidateQueries();
    },
  });

  const createAssetDirectUploadMutation = useMutation({
    mutationFn: async (input: CreateAssetUploadInput) => {
      const data = await graphQLContext?.CreateAssetUpload({
        input,
      });

      if (data?.createAssetUpload) {
        return {
          token: data.createAssetUpload.token,
          url: data.createAssetUpload.url,
          contentType: data.createAssetUpload.contentType,
          contentLength: data.createAssetUpload.contentLength,
          contentEncoding: data.createAssetUpload.contentEncoding,
          next: data.createAssetUpload.next,
        };
      }
    },
  });

  return {
    useGetAssetsQuery,
    createAssetWithStandardUploadMutation,
    createAssetDirectUploadMutation,
    updateAssetMutation,
    deleteAssetMutation,
  };
};
