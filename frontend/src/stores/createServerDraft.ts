import { defineStore } from "pinia";

export type ServerSourceType = "archive" | "folder" | "";

export interface CreateServerDraft {
  sourcePath: string;
  sourceType: ServerSourceType;
}

interface CreateServerDraftState {
  draft: CreateServerDraft | null;
}

export const useCreateServerDraftStore = defineStore("createServerDraft", {
  state: (): CreateServerDraftState => ({
    draft: null,
  }),
  actions: {
    setDraft(payload: CreateServerDraft) {
      this.draft = payload;
    },
    consumeDraft(): CreateServerDraft | null {
      const value = this.draft;
      this.draft = null;
      return value;
    },
    clearDraft() {
      this.draft = null;
    },
  },
});
