import { useQuery } from "@tanstack/react-query";

import { fetchAndReadData } from "@flow/utils/fetchAndReadGeoData";

export const useDebugRunUrlQuery = (dataUrl: string) => {
  return useQuery({
    queryKey: ["dataUrl", dataUrl],
    queryFn: () => {
      return fetchAndReadData(dataUrl);
    },
    staleTime: 5 * 60 * 1000,
    gcTime: 10 * 60 * 1000,
    retry: 2,
    refetchOnWindowFocus: false,
  });
};
