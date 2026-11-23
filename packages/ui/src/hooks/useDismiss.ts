import { useEffect, useRef, type RefObject } from "react";

/**
 * Open popovers and dialogs, oldest first. Only the newest one answers a click
 * outside or Escape, so a confirmation over a dialog (or a menu inside one)
 * closes alone instead of taking the dialog with it.
 */
const layers: symbol[] = [];

/**
 * Calls `onDismiss` when the user presses outside `ref` or hits Escape, while
 * `enabled` and while it's the top-most open layer. For popovers, menus,
 * dropdowns and dialogs.
 */
export function useDismiss(
  ref: RefObject<HTMLElement | null>,
  onDismiss: () => void,
  enabled = true,
): void {
  const callback = useRef(onDismiss);
  useEffect(() => {
    callback.current = onDismiss;
  });

  useEffect(() => {
    if (!enabled) return;
    const me = Symbol("layer");
    layers.push(me);
    const onTop = () => layers[layers.length - 1] === me;
    const onPointerDown = (e: PointerEvent) => {
      const el = ref.current;
      if (onTop() && el && e.target instanceof Node && !el.contains(e.target)) callback.current();
    };
    const onKeyDown = (e: KeyboardEvent) => {
      if (onTop() && e.key === "Escape") callback.current();
    };
    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
      const i = layers.indexOf(me);
      if (i !== -1) layers.splice(i, 1);
    };
  }, [ref, enabled]);
}
