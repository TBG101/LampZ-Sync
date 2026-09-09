function Input({ className = "", ...props }) {
    return (
        <input
            autoComplete="off"
            autoCorrect="off"
            autoCapitalize="off"
            spellCheck={false}
            data-lpignore="true"
            className={`h-11 w-full rounded-none border border-line bg-input px-3 text-sm text-ink outline-none transition-colors placeholder:text-muted/70 hover:border-input-hover focus:border-accent focus:ring-2 focus:ring-accent/20 ${className}`}
            {...props}
        />
    );
}

export default Input;