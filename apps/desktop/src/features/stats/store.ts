import { create } from "zustand";
import { subscribeTimer } from "../focus/api";
import { onPlanChanged } from "../plan/api";
import { statsApi, type ExportFormat, type StatsRange, type StatsView } from "./api";

type StatsState = {
  range: StatsRange;
  view: StatsView | null;
  error: string | null;
  /** The last export's path, shown as a confirmation. */
  exported: string | null;
  /** Loads the view and refreshes it when sessions change. Returns a cleanup. */
  connect: () => () => void;
  setRange: (range: StatsRange) => Promise<void>;
  refresh: () => Promise<void>;
  exportSessions: (format: ExportFormat) => Promise<void>;
};

export const useStatsStore = create<StatsState>((set, get) => ({
  range: "week",
  view: null,
  error: null,
  exported: null,
  connect: () => {
    void get().refresh();
    const offPlan = onPlanChanged(() => void get().refresh());
    const offTimer = subscribeTimer({
      onTick: () => {},
      onPhase: (e) => {
        if (e.type === "focusEnded") void get().refresh();
      },
    });
    return () => {
      offPlan();
      offTimer();
    };
  },
  setRange: async (range) => {
    set({ range });
    await get().refresh();
  },
  refresh: async () => {
    const range = get().range;
    try {
      const view = await statsApi.view(range);
      // Ignore a late answer for a range that's no longer picked.
      if (get().range === range) set({ view, error: null });
    } catch (e) {
      set({ error: e instanceof Error ? e.message : String(e) });
    }
  },
  exportSessions: async (format) => {
    try {
      set({ exported: await statsApi.exportSessions(format), error: null });
    } catch (e) {
      set({ error: e instanceof Error ? e.message : String(e) });
    }
  },
}));
