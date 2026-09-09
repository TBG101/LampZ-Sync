import type { ReactNode } from "react";

function Panel({
    index,
    title,
    sticky,
    className = "",
    children,
}: {
    index: string;
    title: string;
    sticky?: boolean;
    className?: string;
    children: ReactNode;
}) {
    return (
        <div
            className={`border border-line bg-panel p-6 shadow-panel md:p-8 ${sticky ? "sticky top-6 max-md:static" : ""} ${className}`}
        >
            <div className="flex items-start gap-3.5 border-b border-line pb-5">
                <span className="font-mono text-xs font-bold leading-tight text-accent">{index}</span>
                <div>
                    <h2 className="text-xl tracking-[-.02em]">{title}</h2>
                </div>
            </div>
            {children}
        </div>
    );
}

export default Panel;
