import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { tradingMorning, tradingSettings } from "../../test/tradingFixture";
import { TradingScreen } from "./TradingScreen";
import { useTradingStore } from "./store";

const api = vi.hoisted(() => ({
  tradingApi: {
    view: vi.fn(),
    settings: vi.fn(),
    setSettings: vi.fn(),
    setChecked: vi.fn(),
    addItem: vi.fn(),
    removeItem: vi.fn(),
    resetChecklist: vi.fn(),
    restoreSuggested: vi.fn(),
    headCheck: vi.fn(),
    clearHeadCheck: vi.fn(),
    openAutomationSettings: vi.fn(),
  },
  onTradingChanged: () => () => {},
}));
vi.mock("./api", () => api);

describe("TradingScreen", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useTradingStore.setState({ view: null, settings: null, error: null });
    api.tradingApi.view.mockResolvedValue(tradingMorning);
    api.tradingApi.settings.mockResolvedValue(tradingSettings);
    for (const fn of [
      "setChecked",
      "addItem",
      "removeItem",
      "headCheck",
      "clearHeadCheck",
      "setSettings",
    ] as const) {
      api.tradingApi[fn].mockResolvedValue(null);
    }
  });

  it("shows the clocks, the gate, the checklist, the head check and chart time", async () => {
    render(<TradingScreen />);
    expect(await screen.findByText("4h 02m 00s")).toBeInTheDocument();
    expect(screen.getByText("Your window opens in")).toBeInTheDocument();
    expect(screen.getByRole("status", { name: "Gate: 2 steps to unlock" })).toBeInTheDocument();
    expect(screen.getByText("Lagos 11:28:00")).toBeInTheDocument();
    expect(screen.getByText("· your window")).toBeInTheDocument();
    expect(screen.getByText("15:30–22:00 Lagos")).toBeInTheDocument();
    expect(screen.getByText("1 of 2 done · resets every morning")).toBeInTheDocument();
    expect(screen.getByText("How clear is your head right now?")).toBeInTheDocument();
    const chart = screen.getByRole("region", { name: "Chart time outside your window" });
    expect(within(chart).getAllByText("41m").length).toBeGreaterThan(0);
    expect(within(chart).getByText("Nudge me after 5 min")).toBeInTheDocument();
    expect(within(chart).getByText("71%")).toBeInTheDocument();
    expect(within(chart).getByText("29%")).toBeInTheDocument();
    expect(
      within(chart).getByRole("img", {
        name: "Share of outside time: TradingView 71%, DEX Screener 29%",
      }),
    ).toBeInTheDocument();
    // Settings live in a dialog, not on the screen.
    expect(screen.queryByRole("dialog", { name: "Desk settings" })).not.toBeInTheDocument();
  });

  it("ticks items, adds rules, and logs a head check with factors", async () => {
    render(<TradingScreen />);
    await screen.findByText("4h 02m 00s");

    const checklist = screen.getByRole("region", { name: "Pre-trade checklist" });
    fireEvent.click(within(checklist).getByRole("button", { name: /^Key levels marked/ }));
    expect(api.tradingApi.setChecked).toHaveBeenCalledWith("b", true);

    fireEvent.change(screen.getByLabelText("Add a checklist item"), {
      target: { value: "No revenge trades" },
    });
    fireEvent.click(within(checklist).getByRole("button", { name: "Add" }));
    expect(api.tradingApi.addItem).toHaveBeenCalledWith("No revenge trades", "");

    fireEvent.click(screen.getByRole("button", { name: "Calm" }));
    fireEvent.click(screen.getByRole("button", { name: "4, Clear" }));
    expect(api.tradingApi.headCheck).toHaveBeenCalledWith(4, ["Calm"]);
  });

  it("asks before removing a checklist rule", async () => {
    render(<TradingScreen />);
    await screen.findByText("4h 02m 00s");

    fireEvent.click(screen.getByRole("button", { name: "Remove Key levels marked" }));
    const dialog = screen.getByRole("alertdialog", { name: "Remove this rule?" });
    expect(within(dialog).getByText(/Key levels marked/)).toBeInTheDocument();
    expect(within(dialog).getByRole("button", { name: "Cancel" })).toHaveFocus();
    fireEvent.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(api.tradingApi.removeItem).not.toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "Remove Key levels marked" }));
    fireEvent.click(screen.getByRole("button", { name: "Remove" }));
    expect(api.tradingApi.removeItem).toHaveBeenCalledWith("b");
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
  });

  it("takes back a head check with a second click", async () => {
    api.tradingApi.view.mockResolvedValue({
      ...tradingMorning,
      headCheck: { score: 4, factors: ["Calm"], atLabel: "Logged at 09:12", lockedToday: false },
    });
    render(<TradingScreen />);
    const four = await screen.findByRole("button", { name: "4, Clear" });
    expect(four).toHaveAttribute("aria-pressed", "true");
    fireEvent.click(four);
    expect(api.tradingApi.clearHeadCheck).toHaveBeenCalled();
    expect(api.tradingApi.headCheck).not.toHaveBeenCalled();
  });

  it("asks before removing a charting source, and edits one", async () => {
    api.tradingApi.setSettings.mockImplementation(async (s: unknown) => s);
    render(<TradingScreen />);
    await screen.findByText("4h 02m 00s");
    await vi.waitFor(() => expect(api.tradingApi.settings).toHaveBeenCalled());
    fireEvent.click(screen.getByRole("button", { name: "Desk settings" }));
    const dialog = await screen.findByRole("dialog", { name: "Desk settings" });

    // Remove asks first; Cancel keeps it.
    fireEvent.click(within(dialog).getByRole("button", { name: "Stop tracking Binance" }));
    const confirm = screen.getByRole("alertdialog", { name: "Stop tracking this?" });
    expect(within(confirm).getByText(/Binance/)).toBeInTheDocument();
    fireEvent.click(within(confirm).getByRole("button", { name: "Cancel" }));
    expect(api.tradingApi.setSettings).not.toHaveBeenCalled();
    expect(screen.getByRole("dialog", { name: "Desk settings" })).toBeInTheDocument();

    fireEvent.click(within(dialog).getByRole("button", { name: "Stop tracking Binance" }));
    fireEvent.click(screen.getByRole("button", { name: "Remove" }));
    expect(api.tradingApi.setSettings).toHaveBeenLastCalledWith({
      ...tradingSettings,
      sources: [tradingSettings.sources[0], tradingSettings.sources[2]],
    });

    // Edit DEX Screener: add an app, keep the site.
    fireEvent.click(await within(dialog).findByRole("button", { name: "Edit DEX Screener" }));
    const form = within(dialog).getByRole("form", { name: "Edit DEX Screener" });
    fireEvent.change(within(form).getByLabelText("Apps"), {
      target: { value: "DEX Screener, Dexscreener" },
    });
    fireEvent.click(within(form).getByRole("button", { name: "Save" }));
    expect(api.tradingApi.setSettings).toHaveBeenLastCalledWith({
      ...tradingSettings,
      sources: [
        tradingSettings.sources[0],
        {
          label: "DEX Screener",
          apps: ["DEX Screener", "Dexscreener"],
          hosts: ["dexscreener.com"],
        },
      ],
    });
  });

  it("shows an empty checklist, and can bring the suggested rules back", async () => {
    api.tradingApi.view.mockResolvedValue({
      ...tradingMorning,
      checklist: [],
      checklistDone: 0,
      checklistSummary: "Add the rules you check before trading",
    });
    api.tradingApi.restoreSuggested.mockResolvedValue(null);
    render(<TradingScreen />);
    expect(await screen.findByText("No checklist yet")).toBeInTheDocument();
    expect(screen.queryByRole("list", { name: "Checklist rules" })).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Use suggested rules" }));
    expect(api.tradingApi.restoreSuggested).toHaveBeenCalled();
  });

  it("locks the head check after a low score", async () => {
    api.tradingApi.view.mockResolvedValue({
      ...tradingMorning,
      headCheck: { score: 2, factors: [], atLabel: "Logged at 09:12", lockedToday: true },
      gate: { state: "sitOut", label: "Sitting out today", detail: "Rest.", halfSize: false },
    });
    const onFocusBlock = vi.fn();
    render(<TradingScreen onFocusBlock={onFocusBlock} />);
    expect(await screen.findByText("Sit this session out")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "5, Sharp" })).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "Start a focus block instead" }));
    expect(onFocusBlock).toHaveBeenCalled();
  });

  it("opens desk settings from the header and from chart time", async () => {
    render(<TradingScreen />);
    await screen.findByText("4h 02m 00s");
    await vi.waitFor(() => expect(api.tradingApi.settings).toHaveBeenCalled());

    // Saving returns the saved settings, like Rust does.
    api.tradingApi.setSettings.mockImplementation(async (s: unknown) => s);
    fireEvent.click(screen.getByRole("button", { name: "Desk settings" }));
    const dialog = await screen.findByRole("dialog", { name: "Desk settings" });
    fireEvent.click(within(dialog).getByRole("button", { name: "Sat" }));
    expect(api.tradingApi.setSettings).toHaveBeenCalledWith({
      ...tradingSettings,
      days: [true, true, true, true, true, true, false],
    });
    expect(within(dialog).getByText(/Only apps are counted/)).toBeInTheDocument();
    fireEvent.click(within(dialog).getByRole("button", { name: "Done" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();

    fireEvent.click(
      screen.getByRole("button", { name: "Tracked from TradingView, Binance and DEX Screener" }),
    );
    expect(await screen.findByRole("dialog", { name: "Desk settings" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Stop tracking Binance" })).toBeInTheDocument();
  });
});
