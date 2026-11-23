import { useEffect, useMemo, useRef, useState } from "react";
import { appApi } from "../../features/app/api";
import { useFocusStore } from "../../features/focus/store";
import { presetOf } from "../../features/focus/format";
import { planApi } from "../../features/plan/api";
import { usePlanStore } from "../../features/plan/store";
import { filterItems, paletteItems, type PaletteAction } from "../../features/palette/commands";

/** The ⌘K command palette. */
export function PaletteWindow() {
  const { snapshot, connect } = useFocusStore();
  const actions = useFocusStore();
  const inputRef = useRef<HTMLInputElement>(null);
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);
  const [miniShown, setMiniShown] = useState(false);

  useEffect(() => connect(), [connect]);
  const plan = usePlanStore((st) => st.view);
  const connectPlan = usePlanStore((st) => st.connect);
  useEffect(() => connectPlan(), [connectPlan]);
  const todayTasks = useMemo(
    () =>
      (plan?.today ?? [])
        .filter((t) => !t.parentId && !t.completed)
        .map((t) => ({ id: t.id, title: t.title })),
    [plan],
  );

  // Each time the palette opens: fresh query, focused input, current mini state.
  useEffect(() => {
    const onOpen = () => {
      setQuery("");
      setActive(0);
      inputRef.current?.focus();
      void appApi
        .miniVisible()
        .then(setMiniShown)
        .catch(() => {});
    };
    onOpen();
    window.addEventListener("focus", onOpen);
    return () => window.removeEventListener("focus", onOpen);
  }, []);

  const items = useMemo(
    () => filterItems(paletteItems(snapshot, miniShown, todayTasks), query, snapshot),
    [snapshot, miniShown, query, todayTasks],
  );

  const run = async (action: PaletteAction) => {
    await appApi.hidePalette().catch(() => {});
    switch (action.type) {
      case "start":
        return actions.start(action.preset, action.intention);
      case "startPending":
        return actions.startPending();
      case "pause":
        return actions.pause();
      case "resume":
        return actions.resume();
      case "extend":
        return actions.extend(5);
      case "end":
        return actions.end();
      case "toggleMini":
        return appApi.setMiniVisible(!miniShown);
      case "openMain":
        return appApi.openMainWindow();
      case "openSettings":
        return appApi.requestUi("openSettings");
      case "logSession":
        return appApi.requestUi("logSession");
      case "startTask":
        return planApi.startTask(action.taskId, snapshot ? presetOf(snapshot) : "pomodoro");
      case "addTask":
        return planApi.quickAdd(action.text);
    }
  };

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      setActive((i) => Math.min(i + 1, items.length - 1));
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      setActive((i) => Math.max(i - 1, 0));
    } else if (e.key === "Enter") {
      e.preventDefault();
      const item = items[active];
      if (item) void run(item.action);
    } else if (e.key === "Escape") {
      e.preventDefault();
      void appApi.hidePalette();
    }
  };

  return (
    <div className="p-1">
      <div className="overflow-hidden rounded-popover border border-white/10 bg-glass-popover text-text shadow-popover backdrop-blur-[40px] backdrop-saturate-150">
        <div className="flex items-center gap-3 border-b border-hairline px-4">
          <svg
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="var(--halo-text-muted)"
            strokeWidth="2"
            strokeLinecap="round"
            aria-hidden="true"
          >
            <circle cx="11" cy="11" r="7" />
            <path d="m20 20-3.5-3.5" />
          </svg>
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setActive(0);
            }}
            onKeyDown={onKeyDown}
            placeholder="Type a command, or “focus 50 on …”"
            aria-label="Command"
            aria-controls="palette-list"
            aria-activedescendant={items[active] ? `palette-${items[active].id}` : undefined}
            autoComplete="off"
            spellCheck={false}
            className="h-14 flex-1 bg-transparent text-[17px] outline-none placeholder:text-text-muted select-text"
          />
        </div>
        <ul
          id="palette-list"
          role="listbox"
          aria-label="Commands"
          className="max-h-[320px] overflow-auto p-1.5"
        >
          {items.length === 0 && (
            <li className="px-3 py-3 text-[13px] text-text-muted">No matching command</li>
          )}
          {items.map((item, i) => (
            <li
              key={item.id}
              id={`palette-${item.id}`}
              role="option"
              aria-selected={i === active}
              onPointerEnter={() => setActive(i)}
              onClick={() => void run(item.action)}
              className={`flex h-11 cursor-pointer items-center justify-between gap-4 rounded-control px-3 text-[14px] ${
                i === active ? "bg-white/8 text-text" : "text-text-secondary"
              } ${item.id === "quick-start" ? "font-medium text-focus-soft" : ""}`}
            >
              <span className="truncate">{item.label}</span>
              <span className="shrink-0 text-xs text-text-muted">{item.hint}</span>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
