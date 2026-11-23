import { useEffect, useId, useRef, useState, type KeyboardEvent } from "react";
import { useAnchoredPopover } from "../hooks/useAnchoredPopover";
import { useDismiss } from "../hooks/useDismiss";
import { parseTime } from "../time";

type TimePickerProps = {
  label: string;
  /** 24-hour `HH:MM`, or "" for no time. */
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  /** Minutes between the suggested times in the list. */
  step?: number;
};

/** A 24-hour time field: type a time, or pick one from the list. */
export function TimePicker({
  label,
  value,
  onChange,
  placeholder = "--:--",
  step = 15,
}: TimePickerProps) {
  const id = useId();
  const rootRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLUListElement>(null);
  const [open, setOpen] = useState(false);
  const [text, setText] = useState(value);
  const [active, setActive] = useState(-1);
  const [invalid, setInvalid] = useState(false);

  // Follow a new value from outside (state during render, not an effect).
  const [seen, setSeen] = useState(value);
  if (seen !== value) {
    setSeen(value);
    setText(value);
  }

  const slots = Array.from({ length: Math.ceil(1440 / step) }, (_, i) => {
    const m = i * step;
    return `${String(Math.floor(m / 60)).padStart(2, "0")}:${String(m % 60).padStart(2, "0")}`;
  });

  /** The slot at or just before `hhmm`. */
  const slotOf = (hhmm: string) => {
    const [h = 0, m = 0] = hhmm.split(":").map(Number);
    return Math.min(slots.length - 1, Math.max(0, Math.floor((h * 60 + m) / step)));
  };

  /**
   * Scrolls the list (only the list: `scrollIntoView` would also scroll the
   * dialog, and a scroll outside the list closes it) to show slot `i`.
   */
  const reveal = (i: number, how: "center" | "nearest") => {
    const list = listRef.current;
    const el = list?.children[i];
    if (!list || !(el instanceof HTMLElement)) return;
    const top = el.offsetTop;
    const bottom = top + el.offsetHeight;
    if (how === "center") list.scrollTop = top - (list.clientHeight - el.offsetHeight) / 2;
    else if (top < list.scrollTop) list.scrollTop = top;
    else if (bottom > list.scrollTop + list.clientHeight)
      list.scrollTop = bottom - list.clientHeight;
  };

  const close = () => {
    setOpen(false);
    setActive(-1);
  };
  useDismiss(rootRef, close, open);
  useAnchoredPopover(open, inputRef, listRef, close, { maxHeight: 240 });

  // Open on the chosen time, or the nearest slot before it.
  useEffect(() => {
    if (!open) return;
    const target = parseTime(text) || value || "09:00";
    const i = slotOf(target);
    reveal(i, "center");
    // Only when it opens.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  const commit = (raw: string) => {
    const parsed = parseTime(raw);
    if (parsed === null) {
      setInvalid(true);
      return;
    }
    setInvalid(false);
    setText(parsed);
    if (parsed !== value) onChange(parsed);
  };

  const pick = (slot: string) => {
    setInvalid(false);
    setText(slot);
    onChange(slot);
    close();
  };

  const onKey = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
      e.preventDefault();
      if (!open) setOpen(true);
      const from = active >= 0 ? active : slotOf(parseTime(text) || value || "09:00");
      const next = Math.min(slots.length - 1, Math.max(0, from + (e.key === "ArrowDown" ? 1 : -1)));
      setActive(next);
      reveal(next, "nearest");
    } else if (e.key === "Enter") {
      // Keep Enter from submitting the dialog while picking.
      if (open) e.preventDefault();
      const slot = slots[active];
      if (open && slot) pick(slot);
      else {
        commit(text);
        close();
      }
    } else if (e.key === "Escape" && open) {
      e.preventDefault();
      e.stopPropagation();
      e.nativeEvent.stopImmediatePropagation();
      setText(value);
      close();
    } else if (e.key === "Tab") {
      close();
    }
  };

  return (
    <div ref={rootRef} className="relative">
      <input
        ref={inputRef}
        type="text"
        inputMode="numeric"
        autoComplete="off"
        role="combobox"
        aria-label={label}
        aria-expanded={open}
        aria-controls={`${id}-list`}
        aria-invalid={invalid || undefined}
        aria-activedescendant={open && active >= 0 ? `${id}-slot-${active}` : undefined}
        placeholder={placeholder}
        value={text}
        onChange={(e) => {
          setText(e.target.value);
          setInvalid(false);
        }}
        onFocus={() => setOpen(true)}
        onClick={() => setOpen(true)}
        onBlur={() => commit(text)}
        onKeyDown={onKey}
        className={`h-11 w-full min-w-0 rounded-control border bg-control px-3 text-sm text-text tabular-nums outline-none select-text ${
          invalid ? "border-alert/70" : "border-white/10 focus:border-focus/60"
        }`}
      />
      {invalid && (
        <p className="mt-1 text-xs text-alert" role="alert">
          Use a time like 15:30
        </p>
      )}
      {open && (
        <ul
          ref={listRef}
          id={`${id}-list`}
          role="listbox"
          aria-label={label}
          // Keep focus in the input while clicking a time.
          onPointerDown={(e) => e.preventDefault()}
          className="fixed z-40 w-32 overflow-auto rounded-card border border-white/10 bg-surface p-1 shadow-popover"
        >
          {slots.map((s, i) => {
            const isSelected = s === value;
            return (
              <li
                key={s}
                id={`${id}-slot-${i}`}
                role="option"
                aria-selected={isSelected}
                onPointerEnter={() => setActive(i)}
                onClick={() => pick(s)}
                className={`flex h-9 cursor-pointer items-center rounded-lg px-2.5 text-[13px] tabular-nums ${
                  i === active ? "bg-white/8" : ""
                } ${isSelected ? "font-medium text-focus" : "text-text"}`}
              >
                {s}
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
