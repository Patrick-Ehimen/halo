import { useDismiss } from "@halo/ui";
import { useRef, type FormEvent, type ReactNode } from "react";
import type { PaceView } from "../api";
import { COLORS, PACE } from "./values";

/** A progress ring; `planned` draws today's share lighter, after `percent`. */
export function GoalRing({
  size,
  stroke,
  percent,
  color,
  planned = 0,
  label,
}: {
  size: number;
  stroke: number;
  percent: number;
  color: string;
  planned?: number;
  label?: string;
}) {
  const r = (size - stroke) / 2;
  const c = 2 * Math.PI * r;
  const done = (c * Math.min(percent, 100)) / 100;
  const plan = (c * Math.min(planned, 100 - Math.min(percent, 100))) / 100;
  const mid = size / 2;
  return (
    <svg
      width={size}
      height={size}
      viewBox={`0 0 ${size} ${size}`}
      aria-hidden="true"
      className="shrink-0"
    >
      <circle
        cx={mid}
        cy={mid}
        r={r}
        fill="none"
        stroke="rgba(255,255,255,0.08)"
        strokeWidth={stroke}
      />
      {plan > 0 && (
        <circle
          cx={mid}
          cy={mid}
          r={r}
          fill="none"
          stroke={color}
          strokeOpacity={0.35}
          strokeWidth={stroke}
          strokeLinecap="round"
          strokeDasharray={`${plan} ${c}`}
          strokeDashoffset={-done}
          transform={`rotate(-90 ${mid} ${mid})`}
        />
      )}
      {done > 0 && (
        <circle
          cx={mid}
          cy={mid}
          r={r}
          fill="none"
          stroke={color}
          strokeWidth={stroke}
          strokeLinecap="round"
          strokeDasharray={`${done} ${c}`}
          transform={`rotate(-90 ${mid} ${mid})`}
        />
      )}
      {label && (
        <text
          x={mid}
          y={mid + size * 0.09}
          textAnchor="middle"
          fontSize={size * 0.26}
          fontWeight={600}
          fill="var(--halo-text)"
          style={{ fontVariantNumeric: "tabular-nums" }}
        >
          {label}
        </text>
      )}
    </svg>
  );
}

export function PaceBadge({ pace }: { pace: PaceView }) {
  const p = PACE[pace];
  return (
    <span
      className={`rounded-full px-2.5 py-[3px] text-xs font-medium whitespace-nowrap ${p.className}`}
    >
      {p.label}
    </span>
  );
}

export function Dot({ color, size = 6 }: { color: string; size?: number }) {
  return (
    <span
      aria-hidden="true"
      className="shrink-0 rounded-full"
      style={{ width: size, height: size, background: color }}
    />
  );
}

/** Estimate dots: filled for sessions done. */
export function SessionDots({
  done,
  total,
  color,
}: {
  done: number;
  total: number;
  color: string;
}) {
  const shown = Math.min(total, 8);
  return (
    <span className="flex shrink-0 gap-[3px]" aria-label={`${done} of ${total} sessions`}>
      {Array.from({ length: shown }, (_, i) => (
        <span
          key={i}
          className="size-[7px] rounded-full"
          style={
            i < done
              ? { background: color }
              : { boxSizing: "border-box", border: "1.5px solid rgba(255,255,255,0.25)" }
          }
        />
      ))}
      {total > shown && <span className="text-[11px] text-text-muted">+{total - shown}</span>}
    </span>
  );
}

/** A modal form: Escape or a click outside closes it. */
export function Dialog({
  title,
  children,
  submitLabel,
  onSubmit,
  onClose,
  error,
  danger,
}: {
  title: string;
  children: ReactNode;
  submitLabel: string;
  onSubmit: () => void;
  onClose: () => void;
  error?: string | null;
  danger?: { label: string; onClick: () => void };
}) {
  const ref = useRef<HTMLFormElement>(null);
  useDismiss(ref, onClose);
  const submit = (e: FormEvent) => {
    e.preventDefault();
    onSubmit();
  };
  return (
    <div className="fixed inset-0 z-30 flex items-center justify-center bg-black/50 p-6">
      <form
        ref={ref}
        // Rust checks every value and says what's wrong; the web view's own
        // checks only show a vague "Invalid value".
        noValidate
        role="dialog"
        aria-label={title}
        onSubmit={submit}
        className="max-h-full w-[440px] overflow-y-auto rounded-popover border border-white/10 bg-surface p-5 shadow-popover"
      >
        <h2 className="text-base font-semibold">{title}</h2>
        <div className="mt-4 grid gap-3">{children}</div>
        <p role="alert" className="mt-3 min-h-5 text-[13px] text-alert">
          {error}
        </p>
        <div className="mt-1 flex items-center gap-2">
          {danger && (
            <button
              type="button"
              onClick={danger.onClick}
              className="h-11 rounded-full px-4 text-sm font-medium text-alert hover:bg-alert/10"
            >
              {danger.label}
            </button>
          )}
          <span className="flex-1" />
          <button
            type="button"
            onClick={onClose}
            className="h-11 rounded-full border border-white/10 bg-control px-5 text-sm font-medium text-text"
          >
            Cancel
          </button>
          <button
            type="submit"
            className="h-11 rounded-full bg-focus px-5 text-sm font-semibold text-focus-ink"
          >
            {submitLabel}
          </button>
        </div>
      </form>
    </div>
  );
}

export const textareaClass =
  "min-h-[76px] w-full min-w-0 resize-y rounded-control border border-white/10 bg-control px-3 py-2.5 text-sm leading-normal text-text outline-none select-text focus:border-focus/60";

export const fieldClass =
  "h-11 w-full min-w-0 rounded-control border border-white/10 bg-control px-3 text-sm text-text outline-none select-text focus:border-focus/60";

/**
 * A labelled form control. Custom controls (pickers with popovers) name
 * themselves, so they get a plain wrapper: a `<label>` would pass clicks
 * inside their popover on to their button.
 */
export function Field({
  label,
  children,
  custom = false,
}: {
  label: string;
  children: ReactNode;
  custom?: boolean;
}) {
  const Tag = custom ? "div" : "label";
  return (
    <Tag className="grid content-start gap-1.5 text-[13px] font-medium">
      {label}
      {children}
    </Tag>
  );
}

export function ColorPicker({ value, onChange }: { value: string; onChange: (c: string) => void }) {
  return (
    <div role="radiogroup" aria-label="Colour" className="flex flex-wrap gap-1">
      {COLORS.map((c) => (
        <button
          key={c}
          type="button"
          role="radio"
          aria-checked={value === c}
          aria-label={c}
          onClick={() => onChange(c)}
          className={`flex size-11 items-center justify-center rounded-full border ${
            value === c ? "border-white/60" : "border-transparent"
          }`}
        >
          <span className="size-6 rounded-full" style={{ background: c }} />
        </button>
      ))}
    </div>
  );
}

/** A small text button for card headers. */
export function LinkButton({ children, onClick }: { children: ReactNode; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="min-h-9 rounded-control px-2 text-[13px] font-medium text-focus hover:bg-focus/10"
    >
      {children}
    </button>
  );
}
