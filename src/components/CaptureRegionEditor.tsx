import { useEffect, useRef, useState, type KeyboardEvent, type PointerEvent as ReactPointerEvent } from "react";
import type { CaptureRegion } from "../lib/commands";

type Handle = "nw" | "n" | "ne" | "e" | "se" | "s" | "sw" | "w";
type Interaction =
  | { kind: "draw"; pointerId: number; startX: number; startY: number; region: CaptureRegion }
  | { kind: "move"; pointerId: number; startX: number; startY: number; region: CaptureRegion }
  | { kind: "resize"; pointerId: number; startX: number; startY: number; region: CaptureRegion; handle: Handle };

type CaptureRegionEditorProps = {
  regions: CaptureRegion[];
  monitorWidth: number;
  monitorHeight: number;
  onChange: (regions: CaptureRegion[]) => void;
  onSave: () => void;
  isSaving?: boolean;
  saveState?: "idle" | "saved" | "error";
};

const GRID_SIZE = 0.05;
const EDGE_SNAP_DISTANCE = 0.035;
const MIN_REGION_SIZE = 0.04;

const defaultRegion: CaptureRegion = {
  left: 0.1,
  right: 0.4,
  top: 0.1,
  bottom: 0.4,
};

function clamp(value: number, min = 0, max = 1) {
  return Math.min(max, Math.max(min, value));
}

function normalizeRegion(region: CaptureRegion): CaptureRegion {
  return {
    left: clamp(Math.min(region.left, region.right)),
    right: clamp(Math.max(region.left, region.right)),
    top: clamp(Math.min(region.top, region.bottom)),
    bottom: clamp(Math.max(region.top, region.bottom)),
  };
}

function snapValue(value: number, enabled: boolean) {
  if (!enabled) return clamp(value);

  if (value <= EDGE_SNAP_DISTANCE) return 0;
  if (value >= 1 - EDGE_SNAP_DISTANCE) return 1;

  return clamp(Math.round(value / GRID_SIZE) * GRID_SIZE);
}

function snapRegion(region: CaptureRegion, enabled: boolean): CaptureRegion {
  return normalizeRegion({
    left: snapValue(region.left, enabled),
    right: snapValue(region.right, enabled),
    top: snapValue(region.top, enabled),
    bottom: snapValue(region.bottom, enabled),
  });
}

function keepMinimumRegionSize(region: CaptureRegion): CaptureRegion {
  const left = clamp(region.left, 0, 1 - MIN_REGION_SIZE);
  const top = clamp(region.top, 0, 1 - MIN_REGION_SIZE);
  return {
    left,
    right: Math.max(left + MIN_REGION_SIZE, region.right),
    top,
    bottom: Math.max(top + MIN_REGION_SIZE, region.bottom),
  };
}

function regionsOverlap(first: CaptureRegion, second: CaptureRegion) {
  return first.left < second.right && first.right > second.left && first.top < second.bottom && first.bottom > second.top;
}

function canPlaceRegion(region: CaptureRegion, regions: CaptureRegion[], ignoredIndex = -1) {
  return regions.every((existing, index) => index === ignoredIndex || !regionsOverlap(region, existing));
}

function formatPercent(value: number) {
  return `${Math.round(value * 100)}%`;
}

function CaptureRegionEditor({
  regions,
  monitorWidth,
  monitorHeight,
  onChange,
  onSave,
  isSaving = false,
  saveState = "idle",
}: CaptureRegionEditorProps) {
  const canvasRef = useRef<HTMLDivElement>(null);
  const interactionRef = useRef<Interaction | null>(null);
  const [selectedIndex, setSelectedIndex] = useState<number | null>(regions.length > 0 ? 0 : null);
  const [snapEnabled, setSnapEnabled] = useState(true);

  useEffect(() => {
    setSelectedIndex((current) => (current !== null && current < regions.length ? current : regions.length > 0 ? regions.length - 1 : null));
  }, [regions.length]);

  function getPoint(event: ReactPointerEvent<HTMLElement>) {
    const bounds = canvasRef.current?.getBoundingClientRect();
    if (!bounds || bounds.width === 0 || bounds.height === 0) return null;

    return {
      x: clamp((event.clientX - bounds.left) / bounds.width),
      y: clamp((event.clientY - bounds.top) / bounds.height),
    };
  }

  function updateRegion(index: number, nextRegion: CaptureRegion) {
    const normalizedRegion = normalizeRegion(nextRegion);
    if (!canPlaceRegion(normalizedRegion, regions, index)) return;
    onChange(regions.map((region, regionIndex) => regionIndex === index ? normalizedRegion : region));
  }

  function beginDraw(event: ReactPointerEvent<HTMLDivElement>) {
    if (event.currentTarget !== event.target) return;
    const point = getPoint(event);
    if (!point) return;

    const region = { left: point.x, right: point.x, top: point.y, bottom: point.y };
    interactionRef.current = { kind: "draw", pointerId: event.pointerId, startX: point.x, startY: point.y, region };
    canvasRef.current?.setPointerCapture(event.pointerId);
    onChange([...regions, region]);
    setSelectedIndex(regions.length);
  }

  function beginMove(event: ReactPointerEvent<HTMLDivElement>, index: number) {
    const point = getPoint(event);
    const region = regions[index];
    if (!point || !region) return;

    event.stopPropagation();
    interactionRef.current = { kind: "move", pointerId: event.pointerId, startX: point.x, startY: point.y, region };
    canvasRef.current?.setPointerCapture(event.pointerId);
    setSelectedIndex(index);
  }

  function beginResize(event: ReactPointerEvent<HTMLElement>, index: number, handle: Handle) {
    const point = getPoint(event);
    const region = regions[index];
    if (!point || !region) return;

    event.stopPropagation();
    interactionRef.current = { kind: "resize", pointerId: event.pointerId, startX: point.x, startY: point.y, region, handle };
    canvasRef.current?.setPointerCapture(event.pointerId);
    setSelectedIndex(index);
  }

  function moveInteraction(event: ReactPointerEvent<HTMLDivElement>) {
    const interaction = interactionRef.current;
    const point = getPoint(event);
    if (!interaction || !point || interaction.pointerId !== event.pointerId) return;

    const deltaX = point.x - interaction.startX;
    const deltaY = point.y - interaction.startY;

    if (interaction.kind === "draw") {
      const nextRegion = keepMinimumRegionSize(snapRegion({
        left: Math.min(interaction.startX, point.x),
        right: Math.max(interaction.startX, point.x),
        top: Math.min(interaction.startY, point.y),
        bottom: Math.max(interaction.startY, point.y),
      }, snapEnabled));
      const nextRegions = regions.map((region, index) => index === selectedIndex ? nextRegion : region);
      if (canPlaceRegion(nextRegion, regions, selectedIndex ?? -1)) onChange(nextRegions);
      return;
    }

    if (selectedIndex === null) return;

    if (interaction.kind === "move") {
      const width = interaction.region.right - interaction.region.left;
      const height = interaction.region.bottom - interaction.region.top;
      const left = clamp(interaction.region.left + deltaX, 0, 1 - width);
      const top = clamp(interaction.region.top + deltaY, 0, 1 - height);
      updateRegion(selectedIndex, keepMinimumRegionSize(snapRegion({ left, right: left + width, top, bottom: top + height }, snapEnabled)));
      return;
    }

    const nextRegion = { ...interaction.region };
    if (interaction.handle.includes("w")) nextRegion.left = clamp(interaction.region.left + deltaX);
    if (interaction.handle.includes("e")) nextRegion.right = clamp(interaction.region.right + deltaX);
    if (interaction.handle.includes("n")) nextRegion.top = clamp(interaction.region.top + deltaY);
    if (interaction.handle.includes("s")) nextRegion.bottom = clamp(interaction.region.bottom + deltaY);

    if (nextRegion.right - nextRegion.left < MIN_REGION_SIZE) {
      if (interaction.handle.includes("w")) nextRegion.left = nextRegion.right - MIN_REGION_SIZE;
      else nextRegion.right = nextRegion.left + MIN_REGION_SIZE;
    }
    if (nextRegion.bottom - nextRegion.top < MIN_REGION_SIZE) {
      if (interaction.handle.includes("n")) nextRegion.top = nextRegion.bottom - MIN_REGION_SIZE;
      else nextRegion.bottom = nextRegion.top + MIN_REGION_SIZE;
    }

    updateRegion(selectedIndex, keepMinimumRegionSize(snapRegion(nextRegion, snapEnabled)));
  }

  function finishInteraction(event: ReactPointerEvent<HTMLDivElement>) {
    if (interactionRef.current?.pointerId === event.pointerId) {
      const interaction = interactionRef.current;
      if (interaction.kind === "draw" && selectedIndex !== null) {
        const region = regions[selectedIndex];
        if (!region || region.right - region.left < MIN_REGION_SIZE || region.bottom - region.top < MIN_REGION_SIZE) {
          onChange(regions.filter((_, index) => index !== selectedIndex));
          setSelectedIndex(regions.length > 1 ? Math.max(0, selectedIndex - 1) : null);
        }
      }
      interactionRef.current = null;
      canvasRef.current?.releasePointerCapture(event.pointerId);
    }
  }

  function handleKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    if (event.key === "Delete" || event.key === "Backspace") {
      event.preventDefault();
      deleteSelected();
      return;
    }
    if (event.key !== "Escape") return;

    const interaction = interactionRef.current;
    if (interaction && selectedIndex !== null) {
      if (interaction.kind === "draw") {
        onChange(regions.filter((_, index) => index !== selectedIndex));
        setSelectedIndex(regions.length > 1 ? Math.max(0, selectedIndex - 1) : null);
      } else {
        updateRegion(selectedIndex, interaction.region);
      }
    }
    interactionRef.current = null;
  }

  function deleteSelected() {
    if (selectedIndex === null) return;
    onChange(regions.filter((_, index) => index !== selectedIndex));
    setSelectedIndex((current) => current !== null && current > 0 ? current - 1 : regions.length > 1 ? 0 : null);
  }

  function addRegion() {
    const candidates = [
      defaultRegion,
      { left: 0.55, right: 0.85, top: 0.1, bottom: 0.4 },
      { left: 0.1, right: 0.4, top: 0.55, bottom: 0.85 },
      { left: 0.55, right: 0.85, top: 0.55, bottom: 0.85 },
    ];
    const nextRegion = candidates
      .map(normalizeRegion)
      .find((candidate) => canPlaceRegion(candidate, regions));

    if (!nextRegion) return;
    onChange([...regions, nextRegion]);
    setSelectedIndex(regions.length);
  }

  function resetRegions() {
    onChange([]);
    setSelectedIndex(null);
  }

  const selectedRegion = selectedIndex === null ? undefined : regions[selectedIndex];
  const aspectRatio = monitorWidth > 0 && monitorHeight > 0 ? monitorWidth / monitorHeight : 16 / 9;

  return (
    <div className="capture-editor">
      <div className="capture-editor-toolbar">
        <div>
          <p className="capture-editor-kicker">Capture map</p>
          <p className="capture-editor-summary">{regions.length} {regions.length === 1 ? "region" : "regions"} · {monitorWidth > 0 ? `${monitorWidth} x ${monitorHeight}` : "Select a monitor"}</p>
        </div>
        <div className="capture-editor-actions">
          <button type="button" className="capture-editor-button" onClick={addRegion} title="Add a centered region">
            <span aria-hidden="true">+</span> Add region
          </button>
          <button type="button" className={`capture-editor-button ${snapEnabled ? "is-active" : ""}`} aria-pressed={snapEnabled} onClick={() => setSnapEnabled((current) => !current)} title="Snap edges to the 5% grid and monitor edges">
            <span aria-hidden="true">⌗</span> Snap
          </button>
          <button type="button" className="capture-editor-icon-button" onClick={deleteSelected} disabled={selectedIndex === null} aria-label="Delete selected region" title="Delete selected region">
            <span aria-hidden="true">×</span>
          </button>
        </div>
      </div>

      <div
        ref={canvasRef}
        className={`capture-editor-canvas ${snapEnabled ? "is-snapping" : ""}`}
        style={{ aspectRatio }}
        tabIndex={0}
        role="application"
        aria-label="Capture region editor"
        onPointerDown={beginDraw}
        onPointerMove={moveInteraction}
        onPointerUp={finishInteraction}
        onPointerCancel={finishInteraction}
        onKeyDown={handleKeyDown}
      >
        {snapEnabled && <div className="capture-editor-grid" aria-hidden="true" />}
        {regions.map((region, index) => {
          const isSelected = index === selectedIndex;
          return (
            <div
              key={`${index}-${region.left}-${region.top}-${region.right}-${region.bottom}`}
              className={`capture-region ${isSelected ? "is-selected" : ""}`}
              style={{ left: `${region.left * 100}%`, top: `${region.top * 100}%`, width: `${(region.right - region.left) * 100}%`, height: `${(region.bottom - region.top) * 100}%` }}
              onPointerDown={(event) => beginMove(event, index)}
              onClick={() => setSelectedIndex(index)}
            >
              <span className="capture-region-label">R{String(index + 1).padStart(2, "0")}</span>
              {isSelected && (["nw", "n", "ne", "e", "se", "s", "sw", "w"] as Handle[]).map((handle) => (
                <span key={handle} className={`capture-region-handle handle-${handle}`} onPointerDown={(event) => beginResize(event, index, handle)} />
              ))}
            </div>
          );
        })}
        {regions.length === 0 && <div className="capture-editor-empty"><span className="capture-editor-crosshair" aria-hidden="true">+</span><strong>Draw a capture region</strong><span>Drag anywhere on the monitor map</span></div>}
      </div>

      <div className="capture-editor-footer">
        <div className="capture-editor-selection" aria-live="polite">
          {selectedRegion ? <><span className="capture-editor-selection-dot" /> Region {String((selectedIndex ?? 0) + 1).padStart(2, "0")} · {formatPercent(selectedRegion.right - selectedRegion.left)} x {formatPercent(selectedRegion.bottom - selectedRegion.top)}</> : "No region selected"}
        </div>
        <div className="capture-editor-footer-actions">
          <button type="button" className="capture-editor-reset" onClick={resetRegions} disabled={regions.length === 0}>Clear all</button>
          <button type="button" className="capture-editor-save" onClick={onSave} disabled={isSaving}>
            {isSaving ? "Saving..." : saveState === "saved" ? "Saved" : saveState === "error" ? "Retry save" : "Save regions"}
          </button>
        </div>
      </div>
    </div>
  );
}

export default CaptureRegionEditor;
