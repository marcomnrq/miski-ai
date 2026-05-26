import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../lib/api";
import type { MeetingSummary, Meeting } from "../lib/types";

export function useMeetings() {
  return useQuery<MeetingSummary[]>({
    queryKey: ["meetings"],
    queryFn: () => api.meetings.list(),
  });
}

export function useMeeting(id: string) {
  return useQuery<Meeting>({
    queryKey: ["meetings", id],
    queryFn: () => api.meetings.get(id),
    enabled: !!id,
  });
}

export function useDeleteMeeting() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => api.meetings.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["meetings"] });
    },
  });
}

export function useRenameMeeting() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, name }: { id: string; name: string }) =>
      api.meetings.rename(id, name),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: ["meetings"] });
      queryClient.invalidateQueries({ queryKey: ["meetings", id] });
    },
  });
}

export function useSaveNotes() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, notes }: { id: string; notes: string }) =>
      api.meetings.saveNotes(id, notes),
    onSuccess: (_, { id }) => {
      queryClient.invalidateQueries({ queryKey: ["meetings", id] });
    },
  });
}