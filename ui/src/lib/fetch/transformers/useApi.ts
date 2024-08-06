import {
  GetTransformer,
  GetTransformers,
  GetTransformerSegregated,
} from "@flow/types";

import { useFetch } from "./useFetch";

export const useTransformer = () => {
  const {
    useGetTransformersFetch,
    useGetTransformersByIdFetch,
    useGetTransformersSegregatedFetch,
  } = useFetch();

  const useGetTransformers = (): GetTransformers => {
    const { data, ...rest } = useGetTransformersFetch();
    return {
      transformers: data,
      ...rest,
    };
  };

  const useGetTransformerById = (id: string): GetTransformer => {
    const { data, ...rest } = useGetTransformersByIdFetch(id);
    return {
      transformer: data,
      ...rest,
    };
  };

  const useGetTransformerSegregated = (): GetTransformerSegregated => {
    const { data, ...rest } = useGetTransformersSegregatedFetch();
    return {
      transformers: data,
      ...rest,
    };
  };

  return {
    useGetTransformers,
    useGetTransformerById,
    useGetTransformerSegregated,
  };
};
