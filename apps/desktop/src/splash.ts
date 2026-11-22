/** Shortest time the launch splash stays up, so the mark can finish drawing. */
const MIN_VISIBLE_MS = 1100;
const FADE_MS = 320;

/**
 * Fades out the launch splash from index.html once the app has what it needs
 * to draw. Safe to call more than once. This is UI timing, not session time.
 */
export function hideSplash(): void {
  const splash = document.getElementById("splash");
  if (!splash || splash.dataset.hiding) return;
  splash.dataset.hiding = "true";
  const wait = Math.max(0, MIN_VISIBLE_MS - performance.now());
  setTimeout(() => {
    splash.classList.add("is-done");
    setTimeout(() => splash.remove(), FADE_MS);
  }, wait);
}
