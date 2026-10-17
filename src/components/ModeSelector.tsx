import type { LampMode } from "../lib/commands";

const OPTIONS: { value: LampMode; label: string }[] = [
    { value: "sync", label: "Sync to screen" },
    { value: "static", label: "Static color" },
];

function ModeSelector({
    value,
    onChange,
}: {
    value: LampMode;
    onChange: (mode: LampMode) => void;
}) {
    return (
        <div className="grid grid-cols-2 gap-1 border border-line bg-background-deep p-1">
            {OPTIONS.map((option) => {
                const isActive = option.value === value;
                return (
                    <button
                        key={option.value}
                        type="button"
                        aria-pressed={isActive}
                        onClick={() => onChange(option.value)}
                        className={`px-3 py-2 text-[12px] font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-accent/40 ${
                            isActive
                                ? "bg-accent text-button-text"
                                : "bg-transparent text-muted hover:text-label"
                        }`}
                    >
                        {option.label}
                    </button>
                );
            })}
        </div>
    );
}

export default ModeSelector;
