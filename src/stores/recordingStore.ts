import { create } from "zustand";

interface RecordingStore {
  sessionName: string;
  notes: string;
  setSessionName: (name: string) => void;
  setNotes: (notes: string) => void;
  reset: () => void;
}

export const useRecordingStore = create<RecordingStore>((set) => ({
  sessionName: `Meeting ${new Date().toLocaleDateString()}`,
  notes: "",
  setSessionName: (name) => set({ sessionName: name }),
  setNotes: (notes) => set({ notes }),
  reset: () =>
    set({
      sessionName: `Meeting ${new Date().toLocaleDateString()}`,
      notes: "",
    }),
}));