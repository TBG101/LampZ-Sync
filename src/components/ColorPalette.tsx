import type { StaticColor } from "../lib/commands";
import SliderControl from "./SliderControl";

const PRESETS: { label: string; hex: string }[] = [
    { label: "Red", hex: "#ff3b30" },
    { label: "Orange", hex: "#ff9500" },
    { label: "Amber", hex: "#ffcc00" },
    { label: "Lime", hex: "#a3e635" },
    { label: "Green", hex: "#34c759" },
    { label: "Teal", hex: "#30d5c8" },
    { label: "Cyan", hex: "#32ade6" },
    { label: "Blue", hex: "#007aff" },
    { label: "Indigo", hex: "#5856d6" },
    { label: "Purple", hex: "#af52de" },
    { label: "Pink", hex: "#ff2d55" },
    { label: "White", hex: "#ffffff" },
];

function ColorPalette({
    value,
    onChange,
}: {
    value: StaticColor;
    onChange: (color: StaticColor) => void;
}) {
    const selectedHex = value.hex.toLowerCase();

    return (
        <div className="grid gap-4">
            <div className="grid grid-cols-6 gap-2" role="group" aria-label="Preset colors">
                {PRESETS.map((preset) => {
                    const isSelected = preset.hex.toLowerCase() === selectedHex;
                    return (
                        <button
                            key={preset.hex}
                            type="button"
                            title={preset.label}
                            aria-label={preset.label}
                            aria-pressed={isSelected}
                            onClick={() => onChange({ ...value, hex: preset.hex })}
                            style={{ backgroundColor: preset.hex }}
                            className={`h-8 w-full border transition focus:outline-none focus:ring-2 focus:ring-accent/40 ${
                                isSelected
                                    ? "border-accent ring-2 ring-accent/50"
                                    : "border-line hover:border-accent/60"
                            }`}
                        />
                    );
                })}
            </div>

            <div className="flex items-center justify-between gap-4">
                <div>
                    <label className="text-[13px] font-semibold text-label" htmlFor="static-custom-color">
                        Custom color
                    </label>
                    <p className="mt-1 text-xs text-muted">Pick any color from the system palette.</p>
                </div>
                <input
                    id="static-custom-color"
                    type="color"
                    value={value.hex}
                    onChange={(event) => onChange({ ...value, hex: event.target.value })}
                    className="h-9 w-12 shrink-0 cursor-pointer border border-line bg-transparent p-1"
                />
            </div>

            <SliderControl
                label="Brightness"
                field="static_brightness"
                min="10"
                max="1000"
                step="10"
                description="Brightness of the static color."
                value={value.brightness}
                onChange={(brightness) => onChange({ ...value, brightness })}
            />
        </div>
    );
}

export default ColorPalette;
