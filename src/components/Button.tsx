import type { ButtonHTMLAttributes, ReactNode } from "react";

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & {
    children?: ReactNode;
};

function Button({ className = "", children, ...props }: ButtonProps) {
    return (
        <button
            className={`mt-1 rounded-md bg-win-accent px-4 py-2 font-semibold text-white transition hover:bg-win-accent-hover focus:outline-none focus:ring-2 focus:ring-win-accent/40 active:translate-y-px ${className}`}
            {...props}
        >
            {children}
        </button>
    );
}

export default Button;
