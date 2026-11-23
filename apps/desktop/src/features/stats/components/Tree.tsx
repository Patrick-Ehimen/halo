import type { TreeStageView } from "../api";

const LEAF = "var(--halo-shield)";
const TRUNK = "#9C7A54";
const BLOSSOM = "var(--halo-focus)";

const LABELS: Record<TreeStageView, string> = {
  seed: "A seed",
  sprout: "A sprout",
  sapling: "A sapling",
  youngTree: "A young tree",
  tree: "A tree",
  bloom: "A tree in bloom",
};

/** What grows while you focus: one stage per quarter of the daily goal. */
export function Tree({
  stage,
  size = 64,
  label = true,
}: {
  stage: TreeStageView;
  size?: number;
  /** Announce the stage to screen readers; off when the text says it already. */
  label?: boolean;
}) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 64 64"
      role={label ? "img" : undefined}
      aria-label={label ? LABELS[stage] : undefined}
      aria-hidden={label ? undefined : true}
      data-stage={stage}
    >
      <ellipse cx="32" cy="57" rx="18" ry="3" fill="rgba(255,255,255,0.08)" />
      {stage === "seed" && <ellipse cx="32" cy="54" rx="4" ry="3" fill={TRUNK} />}
      {stage === "sprout" && (
        <g fill="none" stroke={LEAF} strokeWidth="2.5" strokeLinecap="round">
          <path d="M32 56V46" />
          <path d="M32 48c-5 0-8-3-8-7 5 0 8 3 8 7zM32 46c4 0 7-3 7-6-4 0-7 3-7 6z" fill={LEAF} />
        </g>
      )}
      {stage === "sapling" && (
        <g>
          <path d="M32 56V36" stroke={TRUNK} strokeWidth="3" strokeLinecap="round" />
          <path d="M32 44l-6-5M32 40l6-5" stroke={TRUNK} strokeWidth="2" strokeLinecap="round" />
          <circle cx="25" cy="37" r="5" fill={LEAF} />
          <circle cx="39" cy="33" r="5" fill={LEAF} />
          <circle cx="32" cy="31" r="6" fill={LEAF} />
        </g>
      )}
      {(stage === "youngTree" || stage === "tree" || stage === "bloom") && (
        <g>
          <path
            d={stage === "youngTree" ? "M32 56V30" : "M32 56V24"}
            stroke={TRUNK}
            strokeWidth={stage === "youngTree" ? 3.5 : 4.5}
            strokeLinecap="round"
          />
          {stage === "youngTree" ? (
            <g fill={LEAF}>
              <circle cx="32" cy="26" r="11" />
              <circle cx="23" cy="32" r="7" />
              <circle cx="41" cy="32" r="7" />
            </g>
          ) : (
            <g fill={LEAF}>
              <circle cx="32" cy="20" r="13" />
              <circle cx="20" cy="28" r="9" />
              <circle cx="44" cy="28" r="9" />
              <circle cx="32" cy="33" r="8" />
            </g>
          )}
          {stage === "bloom" && (
            <g fill={BLOSSOM}>
              <circle cx="26" cy="15" r="2.2" />
              <circle cx="38" cy="18" r="2.2" />
              <circle cx="18" cy="27" r="2.2" />
              <circle cx="45" cy="29" r="2.2" />
              <circle cx="31" cy="26" r="2.2" />
              <circle cx="36" cy="34" r="2.2" />
            </g>
          )}
        </g>
      )}
    </svg>
  );
}
