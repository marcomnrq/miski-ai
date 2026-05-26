import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import type { Settings, SettingsPatch } from "../lib/types";

export function useSettings() {
  return useQuery<Settings>({
    queryKey: ["settings"],
    queryFn: () => api.settings.get(),
  });
}

export function useUpdateSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (patch: SettingsPatch) => api.settings.update(patch),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["settings"] });
    },
  });
}