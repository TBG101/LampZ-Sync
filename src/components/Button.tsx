import type { ButtonHTMLAttributes, ReactNode } from "react";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
    children?: ReactNode;
    variant?: "primary";
};

function Button({ className = "", variant, children, ...props }: ButtonProps) {
    const base = "mt-1 rounded-md bg-win-accent px-4 py-2 font-semibold text-white transition hover:bg-win-accent-hover focus:outline-none focus:ring-2 focus:ring-win-accent/40 active:translate-y-px";
    const primary = "mt-1 flex w-full items-center justify-between rounded-none bg-accent px-4 py-2 font-semibold text-button-text transition hover:bg-button-hover";

    return (
        <button
            className={`${variant === "primary" ? primary : base} ${className}`}
            {...props}
        >
            {children}
        </button>
    );
}

export default Button;
