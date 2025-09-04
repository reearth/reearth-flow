import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

import { useDebugRunUrlQuery } from "./useDebugRunUrlQuery";

type Props = {
  dataUrl?: string;
};

export default ({
  dataUrl = "",
}: Props): {
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
  isLoading: boolean;
  error: string | null;
} => {
  const { data, isLoading, error } = useDebugRunUrlQuery(dataUrl);

  return {
    fileContent: data?.fileContent ?? null,
    fileType: data?.type ?? null,
    isLoading,
    error: data?.error || (error?.message ?? null),
  };
};
