import { useQuery } from "@tanstack/react-query";
import { api } from "../lib/api";
import type { SetupStatus } from "../lib/types";

export function useSetup() {
  return useQuery<SetupStatus>({
    queryKey: ["setup"],
    queryFn: () => api.setup.check(),
    staleTime: 0, // Always refetch
  });
}