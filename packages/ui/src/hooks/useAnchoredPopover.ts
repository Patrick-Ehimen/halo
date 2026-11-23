import { useEffect, useLayoutEffect, useRef, type RefObject } from "react";

const MARGIN = 8;

/**
 * Places a `position: fixed` popover under its anchor (or above it, when there
 * is more room there), right-aligned to it and kept inside the window. Fixed
 * positioning means a scrolling dialog or card can't clip it.
 *
 * A `backdrop-filter` or `transform` on an ancestor makes that ancestor the
 * fixed element's frame instead of the window, so the popover is first put at
 * 0,0 to measure where that frame is, then offset by it.
 *
 * Closes the popover when the window resizes or anything outside it scrolls,
 * since it doesn't follow the anchor.
 */
export function useAnchoredPopover(
  open: boolean,
  anchorRef: RefObject<HTMLElement | null>,
  popoverRef: RefObject<HTMLElement | null>,
  onClose: () => void,
  { gap = 6, maxHeight = Infinity }: { gap?: number; maxHeight?: number } = {},
): void {
  const close = useRef(onClose);
  useEffect(() => {
    close.current = onClose;
  });

  useLayoutEffect(() => {
    const anchor = anchorRef.current;
    const pop = popoverRef.current;
    if (!open || !anchor || !pop) return;

    const s = pop.style;
    // Measuring lifts the height cap, which would reset a scrolled list.
    const scrolled = pop.scrollTop;
    s.position = "fixed";
    s.left = "0px";
    s.top = "0px";
    s.minWidth = `${anchor.offsetWidth}px`;
    s.maxHeight = "";
    const origin = pop.getBoundingClientRect();
    const a = anchor.getBoundingClientRect();
    const vw = window.innerWidth;
    const vh = window.innerHeight;

    const below = vh - a.bottom - gap - MARGIN;
    const above = a.top - gap - MARGIN;
    const up = origin.height > below && above > below;
    const room = Math.min(maxHeight, up ? above : below);
    const height = Math.min(origin.height, room);
    const y = up ? a.top - gap - height : a.bottom + gap;
    const x = Math.max(MARGIN, Math.min(a.right - origin.width, vw - origin.width - MARGIN));

    s.left = `${x - origin.left}px`;
    s.top = `${y - origin.top}px`;
    if (Number.isFinite(room)) s.maxHeight = `${room}px`;
    pop.scrollTop = scrolled;
    // Place it when it opens, not on every render: hovering a row re-renders,
    // and the anchor doesn't move while it's open (a scroll or resize closes it).
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  useEffect(() => {
    if (!open) return;
    const onScroll = (e: Event) => {
      if (!(e.target instanceof Node && popoverRef.current?.contains(e.target))) close.current();
    };
    const onResize = () => close.current();
    window.addEventListener("scroll", onScroll, true);
    window.addEventListener("resize", onResize);
    return () => {
      window.removeEventListener("scroll", onScroll, true);
      window.removeEventListener("resize", onResize);
    };
  }, [open, popoverRef]);
}
