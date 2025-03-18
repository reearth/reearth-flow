import { useQuery } from "@tanstack/react-query";

import {
  PossibleSubscriptionKeys,
  SubscriptionKeys,
} from "./useSubscriptionSetup";

export function useSubscription(
  subscriptionKey: PossibleSubscriptionKeys,
  secondaryCacheKey?: string,
  disabled?: boolean,
) {
  return useQuery({
    queryKey: [SubscriptionKeys[subscriptionKey], secondaryCacheKey],
    queryFn: () => undefined,
    gcTime: Infinity,
    staleTime: Infinity,
    enabled: !disabled,
  });
}
