function ToggleSwitch({
    label,
    description,
    checked,
    onChange,
    disabled = false,
}: {
    label: string;
    description?: string;
    checked: boolean;
    onChange: (value: boolean) => void;
    disabled?: boolean;
}) {
    return (
        <div className="flex items-start justify-between gap-4">
            <div>
                <label className="text-[13px] font-semibold text-label" htmlFor="sync-toggle">
                    {label}
                </label>
                {description && <p className="mt-1 text-xs text-muted">{description}</p>}
            </div>
            <button
                id="sync-toggle"
                type="button"
                role="switch"
                aria-checked={checked}
                aria-label={label}
                disabled={disabled}
                onClick={() => onChange(!checked)}
                className={`relative mt-0.5 h-6 w-11 shrink-0 rounded-full border transition-colors focus:outline-none focus:ring-2 focus:ring-accent/40 disabled:cursor-not-allowed disabled:opacity-45 ${
                    checked
                        ? "border-status-online bg-status-online/30"
                        : "border-line bg-background-deep"
                }`}
            >
                <span
                    className={`absolute top-1/2 h-4 w-4 -translate-y-1/2 rounded-full transition-all ${
                        checked
                            ? "left-6 bg-status-online shadow-status-online"
                            : "left-1 bg-muted"
                    }`}
                />
            </button>
        </div>
    );
}

export default ToggleSwitch;
