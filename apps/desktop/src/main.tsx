import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { MainWindow } from "./windows/main/MainWindow";
import "@halo/ui/styles.css";

const root = document.getElementById("root");
if (!root) throw new Error("missing #root");

// Each Tauri window will get its own root component here
// (main, tray-popover, mini, break-overlay, palette), picked by window label.
createRoot(root).render(
  <StrictMode>
    <MainWindow />
  </StrictMode>,
);
