/**
 * Side-by-side columns for items whose times overlap, as in a calendar: each
 * item takes the first free column in its group of overlapping items, and the
 * group splits the width by how many columns it needs.
 */
export function lanes<T extends { start: number; end: number }>(
  items: T[],
): { item: T; lane: number; of: number }[] {
  const sorted = [...items].sort((a, b) => a.start - b.start || b.end - a.end);
  const out: { item: T; lane: number; of: number }[] = [];
  let group: { item: T; lane: number; of: number }[] = [];
  let laneEnds: number[] = [];
  let groupEnd = -1;
  const flush = () => {
    for (const g of group) g.of = laneEnds.length;
    out.push(...group);
    group = [];
    laneEnds = [];
  };
  for (const item of sorted) {
    // A visible height even for a zero-length item, so neighbours don't cover it.
    const end = Math.max(item.end, item.start + 25);
    if (item.start >= groupEnd) flush();
    let lane = laneEnds.findIndex((e) => e <= item.start);
    if (lane === -1) lane = laneEnds.length;
    laneEnds[lane] = end;
    groupEnd = Math.max(groupEnd, end);
    group.push({ item, lane, of: 0 });
  }
  flush();
  return out;
}
