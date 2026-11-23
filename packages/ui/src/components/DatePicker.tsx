import { useId, useRef, useState, type KeyboardEvent } from "react";
import { useAnchoredPopover } from "../hooks/useAnchoredPopover";
import { useDismiss } from "../hooks/useDismiss";

type DatePickerProps = {
  label: string;
  /** `YYYY-MM-DD`, or "" for no date. */
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  /** Show a "Clear" action that sets the value to "". */
  clearable?: boolean;
  className?: string;
};

const WEEKDAYS = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];
const POPOVER_WIDTH = 304;

function parse(value: string): Date | null {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  if (!m) return null;
  const d = new Date(Number(m[1]), Number(m[2]) - 1, Number(m[3]));
  return Number.isNaN(d.getTime()) ? null : d;
}

function format(d: Date): string {
  const mm = String(d.getMonth() + 1).padStart(2, "0");
  const dd = String(d.getDate()).padStart(2, "0");
  return `${d.getFullYear()}-${mm}-${dd}`;
}

function addDays(d: Date, n: number): Date {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate() + n);
}

/** Same day of month in another month, clamped to that month's length. */
function addMonths(d: Date, n: number): Date {
  const last = new Date(d.getFullYear(), d.getMonth() + n + 1, 0).getDate();
  return new Date(d.getFullYear(), d.getMonth() + n, Math.min(d.getDate(), last));
}

function sameDay(a: Date | null, b: Date | null): boolean {
  return !!a && !!b && format(a) === format(b);
}

/** Six Monday-first weeks covering the month of `d`. */
function monthGrid(d: Date): Date[] {
  const first = new Date(d.getFullYear(), d.getMonth(), 1);
  const offset = (first.getDay() + 6) % 7;
  return Array.from({ length: 42 }, (_, i) => addDays(first, i - offset));
}

/** A styled date field: a button that opens a month calendar. */
export function DatePicker({
  label,
  value,
  onChange,
  placeholder = "No date",
  clearable = true,
  className = "",
}: DatePickerProps) {
  const id = useId();
  const rootRef = useRef<HTMLDivElement>(null);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const gridRef = useRef<HTMLDivElement>(null);
  const popRef = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);
  const selected = parse(value);
  const [cursor, setCursor] = useState<Date>(() => selected ?? new Date());
  const today = new Date();

  const close = (focusButton: boolean) => {
    setOpen(false);
    if (focusButton) buttonRef.current?.focus({ preventScroll: true });
  };
  useDismiss(rootRef, () => close(false), open);

  useAnchoredPopover(open, buttonRef, popRef, () => close(false));

  const show = () => {
    setCursor(selected ?? new Date());
    setOpen(true);
    requestAnimationFrame(() => gridRef.current?.focus({ preventScroll: true }));
  };

  const pick = (d: Date | null) => {
    onChange(d ? format(d) : "");
    close(true);
  };

  const onButtonKey = (e: KeyboardEvent) => {
    if (["ArrowDown", "ArrowUp", "Enter", " "].includes(e.key)) {
      e.preventDefault();
      show();
    }
  };

  const onGridKey = (e: KeyboardEvent) => {
    const weekday = (cursor.getDay() + 6) % 7;
    const moves: Record<string, () => Date> = {
      ArrowLeft: () => addDays(cursor, -1),
      ArrowRight: () => addDays(cursor, 1),
      ArrowUp: () => addDays(cursor, -7),
      ArrowDown: () => addDays(cursor, 7),
      Home: () => addDays(cursor, -weekday),
      End: () => addDays(cursor, 6 - weekday),
      PageUp: () => addMonths(cursor, e.shiftKey ? -12 : -1),
      PageDown: () => addMonths(cursor, e.shiftKey ? 12 : 1),
    };
    const move = moves[e.key];
    if (move) {
      e.preventDefault();
      setCursor(move());
    } else if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      pick(cursor);
    } else if (e.key === "Escape") {
      // Close only the calendar, not a dialog that contains it.
      e.preventDefault();
      e.stopPropagation();
      e.nativeEvent.stopImmediatePropagation();
      close(true);
    }
  };

  const monthLabel = cursor.toLocaleDateString(undefined, { month: "long", year: "numeric" });
  const shown = selected
    ? selected.toLocaleDateString(undefined, {
        weekday: "short",
        day: "numeric",
        month: "short",
        year: "numeric",
      })
    : placeholder;

  const navButton =
    "flex h-9 w-9 items-center justify-center rounded-lg text-text-secondary hover:bg-white/8 hover:text-text";
  const footButton =
    "h-9 rounded-lg px-3 text-[13px] font-medium text-text-secondary hover:bg-white/8 hover:text-text";

  return (
    <div ref={rootRef} className={`relative ${className}`}>
      <span id={`${id}-label`} className="sr-only">
        {label}
      </span>
      <button
        ref={buttonRef}
        type="button"
        aria-haspopup="dialog"
        aria-expanded={open}
        aria-labelledby={`${id}-label ${id}-value`}
        onClick={() => (open ? close(false) : show())}
        onKeyDown={onButtonKey}
        className={`flex h-11 w-full min-w-0 items-center justify-between gap-2 rounded-control border px-3 text-sm transition-colors ${
          open ? "border-focus/50 bg-white/8" : "border-white/10 bg-control hover:bg-white/8"
        }`}
      >
        <span
          id={`${id}-value`}
          className={`truncate tabular-nums ${selected ? "text-text" : "text-text-muted"}`}
        >
          {shown}
        </span>
        <svg
          width="14"
          height="14"
          viewBox="0 0 14 14"
          fill="none"
          stroke="currentColor"
          strokeWidth="1.4"
          strokeLinecap="round"
          strokeLinejoin="round"
          aria-hidden="true"
          className="shrink-0 text-text-muted"
        >
          <rect x="1.75" y="2.75" width="10.5" height="9.5" rx="2" />
          <path d="M1.75 5.75h10.5M4.75 1.25v2.5M9.25 1.25v2.5" />
        </svg>
      </button>

      {open && (
        <div
          role="dialog"
          aria-label={label}
          ref={popRef}
          style={{ width: POPOVER_WIDTH }}
          className="fixed z-40 rounded-card border border-white/10 bg-surface p-3 shadow-popover"
        >
          <div className="flex items-center justify-between">
            <button
              type="button"
              aria-label="Previous month"
              onClick={() => setCursor(addMonths(cursor, -1))}
              className={navButton}
            >
              <svg
                width="12"
                height="12"
                viewBox="0 0 12 12"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.6"
                strokeLinecap="round"
                strokeLinejoin="round"
                aria-hidden="true"
              >
                <path d="M7.5 3 4.5 6l3 3" />
              </svg>
            </button>
            <span aria-live="polite" className="text-[13px] font-semibold tabular-nums">
              {monthLabel}
            </span>
            <button
              type="button"
              aria-label="Next month"
              onClick={() => setCursor(addMonths(cursor, 1))}
              className={navButton}
            >
              <svg
                width="12"
                height="12"
                viewBox="0 0 12 12"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.6"
                strokeLinecap="round"
                strokeLinejoin="round"
                aria-hidden="true"
              >
                <path d="M4.5 3 7.5 6l-3 3" />
              </svg>
            </button>
          </div>

          <div className="mt-2 grid grid-cols-7 text-center text-[11px] font-medium text-text-muted">
            {WEEKDAYS.map((w) => (
              <span key={w} className="py-1">
                {w}
              </span>
            ))}
          </div>

          <div
            ref={gridRef}
            role="grid"
            tabIndex={-1}
            aria-label={monthLabel}
            aria-activedescendant={`${id}-day-${format(cursor)}`}
            onKeyDown={onGridKey}
            className="grid grid-cols-7 gap-0.5 outline-none"
          >
            {monthGrid(cursor).map((d) => {
              const inMonth = d.getMonth() === cursor.getMonth();
              const isSelected = sameDay(d, selected);
              const isToday = sameDay(d, today);
              const isActive = sameDay(d, cursor);
              return (
                <button
                  key={format(d)}
                  id={`${id}-day-${format(d)}`}
                  type="button"
                  role="gridcell"
                  tabIndex={-1}
                  aria-selected={isSelected}
                  aria-current={isToday ? "date" : undefined}
                  aria-label={d.toLocaleDateString(undefined, {
                    weekday: "long",
                    day: "numeric",
                    month: "long",
                    year: "numeric",
                  })}
                  onPointerEnter={() => setCursor(d)}
                  onClick={() => pick(d)}
                  className={`relative flex h-9 items-center justify-center rounded-lg text-[13px] tabular-nums transition-colors ${
                    isSelected
                      ? "bg-focus font-semibold text-focus-ink"
                      : isActive
                        ? "bg-white/8 text-text"
                        : inMonth
                          ? "text-text"
                          : "text-text-muted/60"
                  } ${isToday && !isSelected ? "font-semibold text-focus" : ""}`}
                >
                  {d.getDate()}
                  {isToday && !isSelected && (
                    <span
                      className="absolute bottom-1 h-1 w-1 rounded-full bg-focus"
                      aria-hidden="true"
                    />
                  )}
                </button>
              );
            })}
          </div>

          <div className="mt-2 flex items-center justify-between border-t border-white/8 pt-2">
            {clearable ? (
              <button type="button" onClick={() => pick(null)} className={footButton}>
                Clear
              </button>
            ) : (
              <span />
            )}
            <button type="button" onClick={() => pick(new Date())} className={footButton}>
              Today
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
