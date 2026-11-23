import { StrictMode, type ReactElement } from "react";
import { createRoot } from "react-dom/client";
import { CheckInWindow } from "./windows/checkin/CheckInWindow";
import { MainWindow } from "./windows/main/MainWindow";
import { MiniWindow } from "./windows/mini/MiniWindow";
import { BreakOverlayWindow } from "./windows/overlay/BreakOverlayWindow";
import { PaletteWindow } from "./windows/palette/PaletteWindow";
import { PopoverWindow } from "./windows/popover/PopoverWindow";
import "@halo/ui/styles.css";

const root = document.getElementById("root");
if (!root) throw new Error("missing #root");

// index.html stamps the Tauri window label on <html> before this runs.
const WINDOWS: Record<string, () => ReactElement> = {
  main: () => <MainWindow />,
  "tray-popover": () => <PopoverWindow />,
  mini: () => <MiniWindow />,
  palette: () => <PaletteWindow />,
  "check-in": () => <CheckInWindow />,
  // One overlay window per display: break-overlay-0, break-overlay-1, …
  "break-overlay": () => <BreakOverlayWindow />,
};
const label = document.documentElement.dataset.window ?? "main";
const key = label.startsWith("break-overlay-") ? "break-overlay" : label;
const render = WINDOWS[key] ?? WINDOWS.main;

createRoot(root).render(<StrictMode>{render?.()}</StrictMode>);
