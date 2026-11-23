import { useId, useRef, useState, type KeyboardEvent } from "react";
import { useDismiss } from "../hooks/useDismiss";

export type DropdownOption<T extends string> = { value: T; label: string };

type DropdownProps<T extends string> = {
  label: string;
  value: T;
  options: DropdownOption<T>[];
  onChange: (value: T) => void;
};

/** A styled select: a button that opens a listbox. Keyboard: arrows, Home, End, Enter, Escape. */
export function Dropdown<T extends string>({ label, value, options, onChange }: DropdownProps<T>) {
  const id = useId();
  const rootRef = useRef<HTMLDivElement>(null);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const listRef = useRef<HTMLUListElement>(null);
  const [open, setOpen] = useState(false);
  const selected = Math.max(
    0,
    options.findIndex((o) => o.value === value),
  );
  const [active, setActive] = useState(selected);

  const close = (focusButton: boolean) => {
    setOpen(false);
    if (focusButton) buttonRef.current?.focus({ preventScroll: true });
  };
  useDismiss(rootRef, () => close(false), open);

  const show = () => {
    setActive(selected);
    setOpen(true);
    requestAnimationFrame(() => listRef.current?.focus({ preventScroll: true }));
  };

  const pick = (index: number) => {
    const option = options[index];
    if (option) onChange(option.value);
    close(true);
  };

  const onButtonKey = (e: KeyboardEvent) => {
    if (["ArrowDown", "ArrowUp", "Enter", " "].includes(e.key)) {
      e.preventDefault();
      show();
    }
  };

  const onListKey = (e: KeyboardEvent) => {
    const last = options.length - 1;
    const moves: Record<string, number> = {
      ArrowDown: Math.min(active + 1, last),
      ArrowUp: Math.max(active - 1, 0),
      Home: 0,
      End: last,
    };
    if (e.key in moves) {
      e.preventDefault();
      setActive(moves[e.key] ?? active);
    } else if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      pick(active);
    } else if (e.key === "Escape") {
      // Close only the list, not a panel that contains it.
      e.preventDefault();
      e.stopPropagation();
      e.nativeEvent.stopImmediatePropagation();
      close(true);
    } else if (e.key === "Tab") {
      close(false);
    }
  };

  return (
    <div ref={rootRef} className="relative">
      <span id={`${id}-label`} className="sr-only">
        {label}
      </span>
      <button
        ref={buttonRef}
        type="button"
        aria-haspopup="listbox"
        aria-expanded={open}
        aria-labelledby={`${id}-label ${id}-value`}
        onClick={() => (open ? close(false) : show())}
        onKeyDown={onButtonKey}
        className={`flex h-11 min-w-[132px] items-center justify-between gap-3 rounded-control border px-3 text-[13px] font-medium text-text transition-colors ${
          open ? "border-focus/50 bg-white/8" : "border-white/10 bg-control hover:bg-white/8"
        }`}
      >
        <span id={`${id}-value`} className="tabular-nums">
          {options[selected]?.label}
        </span>
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
          className={`text-text-muted transition-transform ${open ? "rotate-180" : ""}`}
        >
          <path d="M3 4.5 6 7.5l3-3" />
        </svg>
      </button>

      {open && (
        <ul
          ref={listRef}
          role="listbox"
          tabIndex={-1}
          aria-labelledby={`${id}-label`}
          aria-activedescendant={`${id}-opt-${active}`}
          onKeyDown={onListKey}
          className="absolute top-full right-0 z-30 mt-1.5 max-h-64 min-w-full overflow-auto rounded-card border border-white/10 bg-surface p-1 shadow-popover outline-none"
        >
          {options.map((o, i) => {
            const isSelected = i === selected;
            return (
              <li
                key={o.value}
                id={`${id}-opt-${i}`}
                role="option"
                aria-selected={isSelected}
                onPointerEnter={() => setActive(i)}
                onClick={() => pick(i)}
                className={`flex h-9 cursor-pointer items-center justify-between gap-4 rounded-lg px-2.5 text-[13px] whitespace-nowrap tabular-nums ${
                  i === active ? "bg-white/8 text-text" : "text-text-secondary"
                } ${isSelected ? "font-medium text-text" : ""}`}
              >
                {o.label}
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 14 14"
                  fill="none"
                  stroke="var(--halo-focus)"
                  strokeWidth="1.8"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  aria-hidden="true"
                  className={isSelected ? "" : "invisible"}
                >
                  <path d="m3 7.5 2.5 2.5L11 4.5" />
                </svg>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
