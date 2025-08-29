import { useDebugRunUrlQuery } from "./useDebugRunUrlQuery";

type Props = {
  dataUrl?: string;
};

export default ({ dataUrl = "" }: Props) => {
  const { data, isLoading, error } = useDebugRunUrlQuery(dataUrl);

  return {
    fileContent: data?.fileContent ?? null,
    fileType: data?.type ?? null,
    isLoading,
    error: data?.error || (error?.message ?? null),
  };
};
