function Input({ className = "", ...props }) {
    return (
        <input
            autoComplete="off"
            autoCorrect="off"
            autoCapitalize="off"
            spellCheck={false}
            data-lpignore="true"
            className={`rounded-md border border-win-border bg-win-surface px-3 py-2 text-win-text 
                outline-none transition focus:border-win-accent focus:ring-2 focus:ring-win-accent/20 ${className}`}
            {...props}
        />
    );
}

export default Input;