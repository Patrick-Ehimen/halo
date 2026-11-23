import type { MarketId } from "./api";

/**
 * Each session's colour on the clocks and timelines. Green stays the colour
 * of "your window" (the card border), so no session uses it.
 */
export const MARKET_COLORS: Record<MarketId, string> = {
  sydney: "#C59CFF",
  tokyo: "#FF9F7A",
  london: "#6CB6FF",
  newYork: "#7FD1C9",
};

/** Timeline rows come by name; this maps them back to a colour. */
export const COLOR_BY_NAME: Record<string, string> = {
  Sydney: MARKET_COLORS.sydney,
  Tokyo: MARKET_COLORS.tokyo,
  London: MARKET_COLORS.london,
  "New York": MARKET_COLORS.newYork,
};
