import { create } from "zustand";
import { onPlanChanged, planApi, type PlanView } from "./api";

type PlanState = {
  view: PlanView | null;
  error: string | null;
  /** Loads the plan and follows changes from any window. Returns a cleanup. */
  connect: () => () => void;
  refresh: () => Promise<void>;
  /** Runs a change and reports a refusal as `error`. Resolves true when it worked. */
  run: (change: () => Promise<unknown>) => Promise<boolean>;
  clearError: () => void;
};

/** The day's plan and goals, as Rust built them. */
export const usePlanStore = create<PlanState>((set, get) => ({
  view: null,
  error: null,
  connect: () => {
    void get().refresh();
    return onPlanChanged(() => void get().refresh());
  },
  refresh: async () => {
    try {
      set({ view: await planApi.view() });
    } catch (e) {
      set({ error: e instanceof Error ? e.message : String(e) });
    }
  },
  run: async (change) => {
    try {
      await change();
      set({ error: null });
      // The plan://changed event refreshes every window; refresh here too in
      // case this window misses it.
      await get().refresh();
      return true;
    } catch (e) {
      set({ error: e instanceof Error ? e.message : String(e) });
      return false;
    }
  },
  clearError: () => set({ error: null }),
}));
