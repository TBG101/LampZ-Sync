import { Crop, Plug, SlidersHorizontal, type LucideIcon } from "lucide-react";

export type TabId = "connection" | "visual" | "capture";

type Tab = {
    id: TabId;
    index: string;
    label: string;
    hint: string;
    icon: LucideIcon;
};

const tabs: Tab[] = [
    {
        id: "connection",
        index: "01",
        label: "Connection",
        hint: "Lamp pairing",
        icon: Plug,
    },
    {
        id: "visual",
        index: "02",
        label: "Visual response",
        hint: "Mode & tuning",
        icon: SlidersHorizontal,
    },
    {
        id: "capture",
        index: "03",
        label: "Capture regions",
        hint: "Screen sampling",
        icon: Crop,
    },
];

function Sidebar({
    active,
    onChange,
}: {
    active: TabId;
    onChange: (id: TabId) => void;
}) {
    return (
        <nav
            aria-label="Sections"
            className="flex w-[212px] shrink-0 flex-col border-r border-line bg-panel max-md:w-[64px]"
        >
            <p className="m-0 px-5 pb-3 pt-6 text-[10px] font-bold uppercase tracking-[.18em] text-section max-md:hidden">
                Sections
            </p>

            <ul className="m-0 flex list-none flex-col gap-1 px-2">
                {tabs.map(({ id, index, label, hint, icon: Icon }) => {
                    const isActive = active === id;

                    return (
                        <li key={id}>
                            <button
                                type="button"
                                onClick={() => onChange(id)}
                                aria-label={label}
                                title={label}
                                aria-current={isActive ? "page" : undefined}
                                className={`group flex w-full items-center gap-3 border-l-2 px-3 py-2.5 text-left transition-colors focus:outline-none focus-visible:ring-1 focus-visible:ring-inset focus-visible:ring-accent max-md:justify-center max-md:px-0 ${
                                    isActive
                                        ? "border-accent bg-accent/10 text-accent"
                                        : "border-transparent text-muted hover:bg-white/5 hover:text-ink"
                                }`}
                            >
                                <Icon
                                    size={18}
                                    strokeWidth={1.75}
                                    className="shrink-0"
                                    aria-hidden="true"
                                />
                                <span className="flex min-w-0 flex-col max-md:hidden">
                                    <span className="text-[13px] font-semibold leading-tight">
                                        {label}
                                    </span>
                                    <span
                                        className={`text-[11px] ${
                                            isActive ? "text-accent/70" : "text-section"
                                        }`}
                                    >
                                        {hint}
                                    </span>
                                </span>
                                <span
                                    className={`ml-auto font-mono text-[10px] font-bold max-md:hidden ${
                                        isActive ? "text-accent/70" : "text-section"
                                    }`}
                                >
                                    {index}
                                </span>
                            </button>
                        </li>
                    );
                })}
            </ul>

            <div className="mt-auto px-5 py-4 max-md:hidden">
                <p className="m-0 font-mono text-[10px] uppercase leading-relaxed tracking-[.12em] text-section">
                    LampZ Sync
                </p>
            </div>
        </nav>
    );
}

export default Sidebar;
