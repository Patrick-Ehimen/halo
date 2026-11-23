import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { mockIPC } from "@tauri-apps/api/mocks";
import { afterEach, beforeEach } from "vitest";

// Event listeners from modules a test doesn't mock (plan://changed, window
// focus) attach to a stand-in IPC instead of a missing Tauri runtime.
beforeEach(() => {
  mockIPC(() => null, { shouldMockEvents: true });
});

afterEach(() => {
  // Not clearMocks(): unmounting unlistens asynchronously and still needs it.
  cleanup();
});
