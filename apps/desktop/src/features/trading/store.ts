import { create } from "zustand";
import { onTradingChanged, tradingApi, type TradingSettings, type TradingView } from "./api";

/** How often the clocks refresh. UI scheduling only: Rust works out the times. */
const REFRESH_MS = 1000;

type TradingState = {
  view: TradingView | null;
  settings: TradingSettings | null;
  error: string | null;
  /** Loads the desk and keeps the clocks current. Returns a cleanup. */
  connect: () => () => void;
  refresh: () => Promise<void>;
  /** Runs an action, then reloads; errors show in `error`. */
  run: (action: () => Promise<unknown>) => Promise<boolean>;
  saveSettings: (s: TradingSettings) => Promise<void>;
};

export const useTradingStore = create<TradingState>((set, get) => ({
  view: null,
  settings: null,
  error: null,
  connect: () => {
    void get().refresh();
    void tradingApi
      .settings()
      .then((settings) => set({ settings }))
      .catch(() => {});
    const timer = setInterval(() => void get().refresh(), REFRESH_MS);
    const off = onTradingChanged(() => {
      void get().refresh();
      void tradingApi
        .settings()
        .then((settings) => set({ settings }))
        .catch(() => {});
    });
    return () => {
      clearInterval(timer);
      off();
    };
  },
  refresh: async () => {
    try {
      set({ view: await tradingApi.view() });
    } catch (e) {
      set({ error: e instanceof Error ? e.message : String(e) });
    }
  },
  run: async (action) => {
    try {
      await action();
      set({ error: null });
      await get().refresh();
      return true;
    } catch (e) {
      set({ error: e instanceof Error ? e.message : String(e) });
      return false;
    }
  },
  saveSettings: async (s) => {
    const previous = get().settings;
    set({ settings: s });
    try {
      set({ settings: await tradingApi.setSettings(s), error: null });
      await get().refresh();
    } catch (e) {
      set({ settings: previous, error: e instanceof Error ? e.message : String(e) });
    }
  },
}));
