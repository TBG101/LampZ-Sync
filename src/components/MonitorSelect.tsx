import { useEffect, useId, useRef, useState, type KeyboardEvent } from "react";
import type { MonitorInfo } from "../lib/commands";

function MonitorSelect({
  monitors,
  value,
  onChange,
}: {
  monitors: MonitorInfo[];
  value: string;
  onChange: (deviceName: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const [highlightedIndex, setHighlightedIndex] = useState(0);
  const containerRef = useRef<HTMLDivElement>(null);
  const listboxId = useId();
  const selectedIndex = monitors.findIndex((monitor) => monitor.device_name === value);
  const activeIndex = highlightedIndex < monitors.length ? highlightedIndex : 0;
  const selectedMonitor = monitors[selectedIndex] ?? monitors[0];

  useEffect(() => {
    function handlePointerDown(event: PointerEvent) {
      if (!containerRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    }

    document.addEventListener("pointerdown", handlePointerDown);
    return () => document.removeEventListener("pointerdown", handlePointerDown);
  }, []);

  useEffect(() => {
    setHighlightedIndex(selectedIndex >= 0 ? selectedIndex : 0);
  }, [selectedIndex]);

  function selectMonitor(index: number) {
    const monitor = monitors[index];
    if (!monitor) return;

    onChange(monitor.device_name);
    setOpen(false);
  }

  function handleKeyDown(event: KeyboardEvent<HTMLButtonElement>) {
    if (monitors.length === 0) return;

    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      setOpen(true);
      setHighlightedIndex((current) => {
        const next = event.key === "ArrowDown" ? current + 1 : current - 1;
        return (next + monitors.length) % monitors.length;
      });
    } else if (event.key === "Home") {
      event.preventDefault();
      setOpen(true);
      setHighlightedIndex(0);
    } else if (event.key === "End") {
      event.preventDefault();
      setOpen(true);
      setHighlightedIndex(monitors.length - 1);
    } else if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      if (open) {
        selectMonitor(activeIndex);
      } else {
        setOpen(true);
      }
    } else if (event.key === "Escape") {
      setOpen(false);
    }
  }

  return (
    <div ref={containerRef} className="relative">
      <button
        type="button"
        className={`flex min-h-11 w-full items-center justify-between border bg-input px-3 py-2 text-left text-sm text-ink outline-none transition-colors ${open ? "border-accent ring-2 ring-accent/20" : "border-line hover:border-input-hover"} ${monitors.length === 0 ? "cursor-not-allowed opacity-60" : ""}`}
        aria-controls={listboxId}
        aria-expanded={open}
        aria-haspopup="listbox"
        disabled={monitors.length === 0}
        onClick={() => setOpen((current) => !current)}
        onKeyDown={handleKeyDown}
      >
        <span className="min-w-0 truncate">
          <span className="block">{selectedMonitor?.name ?? "No monitors found"}</span>
          {selectedMonitor && (
            <span className="mt-0.5 block text-xs text-muted">
              {selectedMonitor.width} x {selectedMonitor.height} · {selectedMonitor.refresh_rate} Hz
            </span>
          )}
        </span>
        <span
          aria-hidden="true"
          className={`ml-3 h-2 w-2 shrink-0 rotate-45 border-b border-r border-accent transition-transform ${open ? "-translate-y-0.5" : ""}`}
          style={open ? { transform: "rotate(225deg)" } : undefined}
        />
      </button>

      {open && monitors.length > 0 && (
        <div
          id={listboxId}
          role="listbox"
          aria-label="Available monitors"
          className="absolute z-20 mt-2 max-h-56 w-full overflow-auto border border-line bg-input p-1 shadow-panel"
        >
          {monitors.map((monitor, index) => {
            const isSelected = monitor.device_name === value;
            const isHighlighted = index === activeIndex;

            return (
              <button
                type="button"
                role="option"
                aria-selected={isSelected}
                key={monitor.device_name}
                className={`flex w-full items-center justify-between gap-3 px-3 py-2.5 text-left text-sm transition-colors ${isHighlighted ? "bg-accent/10 text-accent" : "text-ink hover:bg-accent/5"}`}
                onMouseEnter={() => setHighlightedIndex(index)}
                onClick={() => selectMonitor(index)}
              >
                <span className="min-w-0 truncate">
                  <span className="block">{monitor.name}</span>
                  <span className="mt-0.5 block text-xs text-muted">
                    {monitor.width} x {monitor.height} · {monitor.refresh_rate} Hz
                  </span>
                </span>
                {isSelected && <span className="shrink-0 text-xs text-accent">Selected</span>}
              </button>
            );
          })}
        </div>
      )}
    </div>
  );
}

export default MonitorSelect;
