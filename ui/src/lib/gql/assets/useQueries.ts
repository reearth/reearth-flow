import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { Asset, AssetSortType } from "@flow/types";
import {
  OrderDirection,
  type PaginationOptions,
} from "@flow/types/paginationOptions";
import { isDefined } from "@flow/utils";

import { CreateAssetInput, RemoveAssetInput } from "../__gen__/graphql";
import { toAsset } from "../convert";
import { useGraphQLContext } from "../provider";

export enum AssetQueryKeys {
  GetAssets = "getAssets",
}

export const ASSET_FETCH_RATE = 15;

export const useQueries = () => {
  const graphQLContext = useGraphQLContext();
  const queryClient = useQueryClient();

  const useGetAssetsQuery = (
    workspaceId: string,
    sort?: AssetSortType,
    paginationOptions?: PaginationOptions,
  ) =>
    useQuery({
      queryKey: [AssetQueryKeys.GetAssets, workspaceId],
      queryFn: async () => {
        const data = await graphQLContext?.GetAssets({
          workspaceId: workspaceId ?? "",
          sort: sort ?? AssetSortType.Date,
          pagination: {
            page: paginationOptions?.page ?? 1,
            pageSize: ASSET_FETCH_RATE,
            orderDir: paginationOptions?.orderDir ?? OrderDirection.Desc,
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

  const createAssetMutation = useMutation({
    mutationFn: async (input: CreateAssetInput) => {
      const data = await graphQLContext?.CreateAsset({
        input,
      });

      if (data?.createAsset?.asset) {
        return toAsset(data.createAsset.asset);
      }
    },
    onSuccess: (asset) => {
      queryClient.invalidateQueries({
        queryKey: [AssetQueryKeys.GetAssets, asset],
      });
    },
  });

  const removeAssetMutation = useMutation({
    mutationFn: async (input: RemoveAssetInput) => {
      const data = await graphQLContext?.RemoveAsset({
        input,
      });

      return {
        assetId: data?.removeAsset?.assetId,
      };
    },
    onSuccess: (asset) => {
      queryClient.invalidateQueries({
        queryKey: [AssetQueryKeys.GetAssets, asset],
      });
    },
  });

  return {
    useGetAssetsQuery,
    createAssetMutation,
    removeAssetMutation,
  };
};
