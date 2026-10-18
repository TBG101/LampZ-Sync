import { useEffect, useState, type ChangeEvent } from "react";
import type { McpConfig } from "../lib/commands";
import Button from "./Button";
import Input from "./Input";
import ToggleSwitch from "./ToggleSwitch";

function buildClientConfig(port: number, token: string | null) {
    return JSON.stringify(
        {
            mcp: {
                "lampz-sync": {
                    type: "remote",
                    url: `http://127.0.0.1:${port}/mcp`,
                    enabled: true,
                    ...(token ? { headers: { Authorization: `Bearer ${token}` } } : {}),
                },
            },
        },
        null,
        2,
    );
}

function McpPanel({
    value,
    onSave,
}: {
    value: McpConfig;
    onSave: (mcp: McpConfig) => Promise<void> | void;
}) {
    const [port, setPort] = useState(String(value.port));
    const [token, setToken] = useState(value.token ?? "");
    const [saveState, setSaveState] = useState<"idle" | "saved" | "error">("idle");
    const [copied, setCopied] = useState(false);

    useEffect(() => {
        setPort(String(value.port));
        setToken(value.token ?? "");
    }, [value.port, value.token]);

    const parsedPort = Number.parseInt(port, 10);
    const portValid = Number.isInteger(parsedPort) && parsedPort >= 1 && parsedPort <= 65535;

    async function persist(next: McpConfig) {
        try {
            await onSave(next);
            setSaveState("saved");
        } catch (error) {
            console.error(error);
            setSaveState("error");
        }
    }

    function toggleEnabled(enabled: boolean) {
        setSaveState("idle");
        void persist({ enabled, port: value.port, token: value.token });
    }

    function saveServerSettings() {
        if (!portValid) return;
        setSaveState("idle");

        const trimmedToken = token.trim();
        void persist({
            enabled: value.enabled,
            port: parsedPort,
            token: trimmedToken ? trimmedToken : null,
        });
    }

    async function copyConfig() {
        try {
            await navigator.clipboard.writeText(clientConfig);
            setCopied(true);
            window.setTimeout(() => setCopied(false), 1500);
        } catch (error) {
            console.error(error);
        }
    }

    const url = `http://127.0.0.1:${value.port}/mcp`;
    const clientConfig = buildClientConfig(value.port, value.token);

    return (
        <div className="flex flex-col gap-4 pt-4">
            <p className="m-0 max-w-2xl text-sm leading-relaxed text-muted">
                Let other apps and AI agents control the lamp over a local
                Model&nbsp;Context&nbsp;Protocol (MCP) server. It listens only on the
                loopback interface of this machine.
            </p>

            <section className="grid gap-4 border-b border-line pb-4">
                <ToggleSwitch
                    label={value.enabled ? "MCP server enabled" : "MCP server disabled"}
                    description={
                        value.enabled
                            ? "Agents can connect to the endpoint below."
                            : "No agent can connect while the server is disabled."
                    }
                    checked={value.enabled}
                    onChange={toggleEnabled}
                />
            </section>

            <section className="grid gap-4 border-b border-line pb-4">
                <div className="grid gap-4 sm:grid-cols-2">
                    <div className="grid gap-2">
                        <label className="text-[13px] font-semibold text-label" htmlFor="mcp-port">
                            Port
                        </label>
                        <Input
                            id="mcp-port"
                            inputMode="numeric"
                            value={port}
                            onChange={(event: ChangeEvent<HTMLInputElement>) => {
                                setPort(event.target.value);
                                setSaveState("idle");
                            }}
                        />
                        {!portValid && (
                            <p className="m-0 text-xs text-status-offline">
                                Enter a port between 1 and 65535.
                            </p>
                        )}
                    </div>

                    <div className="grid gap-2">
                        <label className="text-[13px] font-semibold text-label" htmlFor="mcp-token">
                            Bearer token (optional)
                        </label>
                        <Input
                            id="mcp-token"
                            value={token}
                            onChange={(event: ChangeEvent<HTMLInputElement>) => {
                                setToken(event.target.value);
                                setSaveState("idle");
                            }}
                            placeholder="Empty allows any local process"
                        />
                    </div>
                </div>

                <Button
                    type="button"
                    variant="primary"
                    onClick={saveServerSettings}
                    disabled={!portValid}
                    className="disabled:cursor-not-allowed disabled:opacity-45"
                >
                    Save server settings <span aria-hidden="true">→</span>
                </Button>

                {saveState === "saved" && <p className="m-0 text-xs text-status-online">Saved.</p>}
                {saveState === "error" && (
                    <p className="m-0 text-xs text-status-offline">Could not save.</p>
                )}
            </section>

            <section className="grid gap-2">
                <p className="m-0 text-[11px] font-bold uppercase tracking-[.15em] text-section">
                    Endpoint
                </p>
                <code className="block break-all border border-line bg-background-deep px-3 py-2 font-mono text-xs text-ink">
                    {url}
                </code>
            </section>

            <section className="grid gap-2">
                <div className="flex items-center justify-between gap-4">
                    <p className="m-0 text-[11px] font-bold uppercase tracking-[.15em] text-section">
                        Client config
                    </p>
                    <button
                        type="button"
                        onClick={copyConfig}
                        className="cursor-pointer text-xs font-semibold text-accent hover:underline focus:outline-none"
                    >
                        {copied ? "Copied" : "Copy"}
                    </button>
                </div>
                <pre className="m-0 overflow-x-auto border border-line bg-background-deep px-3 py-2 font-mono text-[11px] leading-relaxed text-muted">
                    {clientConfig}
                </pre>
            </section>
        </div>
    );
}

export default McpPanel;
