import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

type TitlebarProps = {
  title?: string;
  className?: string;
  isConnected?: boolean;
};

function Titlebar({ title = "LampZ Sync", className = "", isConnected = false }: TitlebarProps) {
  const [isMaximized, setIsMaximized] = useState(false);

  const appWindow = getCurrentWindow();

  useEffect(() => {
    appWindow.isMaximized().then(setIsMaximized);

    const unlisten = appWindow.onResized(() => {
      appWindow.isMaximized().then(setIsMaximized);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [appWindow]);

  const handleMinimize = async () => {
    await appWindow.minimize();
  };

  const handleToggleMaximize = async () => {
    await appWindow.toggleMaximize();
  };

  const handleClose = async () => {
    await appWindow.close();
  };

  return (
    <div
      data-tauri-drag-region
      className={`
    sticky top-0 z-50
    flex h-11 select-none items-center justify-between
    border-b border-line/20 bg-background-deep/80 shadow-titlebar backdrop-blur-md
    ${className}`}
    >
      <div data-tauri-drag-region className="flex items-center gap-3 pl-4">
        <span className="flex h-6 w-6 items-center justify-center border border-accent/40 bg-accent/10 font-mono text-[10px] font-bold tracking-[-.08em] text-accent">
          LZ
        </span>
        <div data-tauri-drag-region className="flex items-center gap-3">
          <span className="text-[13px] font-semibold tracking-wide text-ink">{title}</span>
        </div>
      </div>

      <div className="flex h-full items-center">
        <div
          className={`mr-3 flex items-center gap-2 whitespace-nowrap text-[10px] font-bold uppercase tracking-[.08em] ${
            isConnected ? "text-accent" : "text-status-offline-text"
          }`}
          aria-live="polite"
        >
          <span
            className={`h-1.5 w-1.5 rounded-full ${
              isConnected
                ? "bg-status-online shadow-status-online"
                : "bg-status-offline shadow-status-offline"
            }`}
          />
          <span>{isConnected ? "Connected" : "Not connected"}</span>
        </div>

        <button
          onClick={handleMinimize}
          className="flex h-full w-11 items-center justify-center text-muted transition-colors hover:bg-white/10 hover:text-ink focus:outline-none focus:ring-1 focus:ring-inset focus:ring-accent active:bg-white/5"
          aria-label="Minimize"
        >
          <svg width="10" height="1" viewBox="0 0 10 1" fill="none">
            <path d="M0 0.5H10" stroke="currentColor" strokeWidth="1" />
          </svg>
        </button>

        <button
          onClick={handleToggleMaximize}
          className="flex h-full w-11 items-center justify-center text-muted transition-colors hover:bg-white/10 hover:text-ink focus:outline-none focus:ring-1 focus:ring-inset focus:ring-accent active:bg-white/5"
          aria-label={isMaximized ? "Restore" : "Maximize"}
        >
          {isMaximized ? (
            <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
              <rect
                x="2"
                y="0"
                width="8"
                height="8"
                stroke="currentColor"
                strokeWidth="1"
              />
              <path d="M0 2V10H8" stroke="currentColor" strokeWidth="1" />
            </svg>
          ) : (
            <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
              <rect
                x="0"
                y="0"
                width="10"
                height="10"
                stroke="currentColor"
                strokeWidth="1"
              />
            </svg>
          )}
        </button>

        <button
          onClick={handleClose}
          className="flex h-full w-11 items-center justify-center text-muted transition-colors hover:bg-status-close hover:text-white focus:outline-none focus:ring-1 focus:ring-inset focus:ring-white active:bg-status-close-active"
          aria-label="Close"
        >
          <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
            <path
              d="M1 1L9 9M9 1L1 9"
              stroke="currentColor"
              strokeWidth="1"
            />
          </svg>
        </button>
      </div>
    </div>
  );
}

export default Titlebar;
