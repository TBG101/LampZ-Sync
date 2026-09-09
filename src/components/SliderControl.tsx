import type { LampTuning } from "../lib/commands";

function SliderControl({
    label,
    field,
    min,
    max,
    step = "1",
    unit = "",
    description,
    value,
    onChange,
}: {
    label: string;
    field: keyof LampTuning;
    min: string;
    max: string;
    step?: string;
    unit?: string;
    description: string;
    value: number;
    onChange: (value: number) => void;
}) {
    return (
        <div className="grid gap-2">
            <div className="flex items-start justify-between gap-4">
                <div>
                    <label className="text-[13px] font-semibold text-[#c9d8d6]" htmlFor={field}>{label}</label>
                    <p className="mt-1 text-xs text-muted">{description}</p>
                </div>
                <output className="whitespace-nowrap border border-value-border px-2 py-1 font-mono text-xs font-bold text-accent" htmlFor={field}>
                    {value} <span className="text-[10px] text-value-unit">{unit}</span>
                </output>
            </div>
            <input
                id={field}
                type="range"
                name={field}
                min={min}
                max={max}
                step={step}
                value={value}
                onChange={(event) => onChange(Number(event.target.value))}
                className="range-input"
            />
            <div className="flex justify-between font-mono text-[10px] text-scale" aria-hidden="true"><span>{min}{unit}</span><span>{max}{unit}</span></div>
        </div>
    );
}

export default SliderControl;
