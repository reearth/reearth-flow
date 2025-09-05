import { useQuery } from "@tanstack/react-query";

import { fetchAndReadData } from "@flow/utils/fetchAndReadGeoData";

export const useDebugRunUrlQuery = (dataUrl: string) => {
  return useQuery({
    queryKey: ["dataUrl", dataUrl],
    queryFn: () => {
      return fetchAndReadData(dataUrl);
    },
    staleTime: Infinity,
    gcTime: Infinity,
    retry: 2,
    refetchOnWindowFocus: false,
  });
};
