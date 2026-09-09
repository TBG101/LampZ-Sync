import { useEffect, useState, type ChangeEvent } from "react";
import "./App.css";
import Button from "./components/Button";
import Input from "./components/Input";
import MonitorSelect from "./components/MonitorSelect";
import Panel from "./components/Panel";
import SliderControl from "./components/SliderControl";
import Titlebar from "./components/Titlebar";
import {
  getConfig,
  getMonitors,
  invokeConnectLamp,
  invokeUpdateLampTuning,
  setMonitor,
  type MonitorInfo,
  type LampTuning,
} from "./lib/commands";
import { startConnectionListen } from "./lib/listeners";

type LampForm = {
  id: string;
  ip: string;
  localKey: string;
};

const defaultLampTuning: LampTuning = {
  poll_interval_ms: 50,
  mailbox_timeout_secs: 1,
  gamma: 2,
  hue_threshold: 2,
  sat_threshold: 10,
  v_threshold: 10,
};

function App() {
  const [lamp, setLamp] = useState<LampForm>({
    id: "",
    ip: "",
    localKey: "",
  });
  const [lampConnection, setLampConnection] = useState(false);
  const [lampTuning, setLampTuning] = useState<LampTuning>(defaultLampTuning);
  const [monitors, setMonitors] = useState<MonitorInfo[]>([]);
  const [selectedMonitor, setSelectedMonitor] = useState("");

  useEffect(() => {
    Promise.all([getConfig(), getMonitors()])
      .then(([config, availableMonitors]) => {
        setMonitors(availableMonitors);
        setLamp({
          id: config.lamp_connection.lamp_id ?? "",
          ip: config.lamp_connection.lamp_ip ?? "",
          localKey: config.lamp_connection.lamp_key ?? "",
        });
        setLampTuning(config.lamp_tuning);

        const savedMonitor = config.monitor_device_name;
        const fallbackMonitor =
          savedMonitor && availableMonitors.some(
            (monitor) => monitor.device_name === savedMonitor,
          )
            ? savedMonitor
            : availableMonitors[0]?.device_name ?? "";

        setSelectedMonitor(fallbackMonitor);
      })
      .catch(console.error);

    let unlisten: (() => void) | undefined;
    startConnectionListen((connection) => {
      setLampConnection(connection);
    }).then(fn => {
      unlisten = fn;
    });

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  function updateField(field: keyof LampForm, event: ChangeEvent<HTMLInputElement>) {
    setLamp((current) => ({
      ...current,
      [field]: event.target.value,
    }));
  }

return (
  <div className="app-background flex h-screen flex-col overflow-hidden text-ink">
    {/* Titlebar stays fixed at the top */}
    <Titlebar />

    {/* Scrollbar lives strictly inside this container below the titlebar */}
    <main className="flex-1 overflow-y-auto px-6 pb-8 pt-5 md:px-12 md:pb-16">
      <header className="mx-auto mb-10 flex max-w-[1180px] items-end justify-between gap-8 max-md:mb-8 max-md:flex-col max-md:items-start">
        <div>
          <p className="mb-3 text-[11px] font-bold uppercase tracking-[.15em] text-accent">
            LampZ Sync / Control center
          </p>
        </div>
        <div
          className={`flex items-center gap-2.5 whitespace-nowrap border px-3.5 py-2.5 text-xs font-bold uppercase tracking-[.08em] ${
            lampConnection
              ? "border-status-online-border text-accent"
              : "border-status-offline-border text-status-offline-text"
          }`}
        >
          <span
            className={`h-2 w-2 rounded-full ${
              lampConnection
                ? "bg-status-online shadow-status-online"
                : "bg-status-offline shadow-status-offline"
            }`}
          />
          <span>{lampConnection ? "Connected" : "Not connected"}</span>
        </div>
      </header>

      <div className="mx-auto grid max-w-[1180px] grid-cols-[minmax(260px,.75fr)_minmax(420px,1.25fr)] items-start gap-5 max-md:grid-cols-1">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            invokeConnectLamp({
              lampId: lamp.id,
              lampIp: lamp.ip,
              lampKey: lamp.localKey,
            });
          }}
        >
          <Panel index="01" title="Device connection" sticky>
            <div className="my-7 grid gap-5">
              <div className="grid gap-2">
                <label className="text-[13px] font-semibold text-label" htmlFor="id">
                  Lamp ID
                </label>
                <Input
                  id="id"
                  name="id"
                  value={lamp.id}
                  onChange={(event: ChangeEvent<HTMLInputElement>) => updateField("id", event)}
                  placeholder="e.g. lamp_12345"
                />
              </div>

              <div className="grid gap-2">
                <label className="text-[13px] font-semibold text-label" htmlFor="ip">
                  IP address
                </label>
                <Input
                  id="ip"
                  name="ip"
                  value={lamp.ip}
                  onChange={(event: ChangeEvent<HTMLInputElement>) => updateField("ip", event)}
                  placeholder="192.168.1.100"
                />
              </div>

              <div className="grid gap-2">
                <label className="text-[13px] font-semibold text-label" htmlFor="localKey">
                  Local key
                </label>
                <Input
                  id="localKey"
                  name="localKey"
                  value={lamp.localKey}
                  onChange={(event: ChangeEvent<HTMLInputElement>) => updateField("localKey", event)}
                  placeholder="Secret key"
                />
              </div>
            </div>

            <Button type="submit" variant="primary">
              Connect lamp <span aria-hidden="true">→</span>
            </Button>
          </Panel>
        </form>

        <form
          onSubmit={(event) => {
            event.preventDefault();
            invokeUpdateLampTuning(lampTuning).catch(console.error);
          }}
        >
          <Panel index="02" title="Visual response">
            <div className="flex flex-col gap-5 pt-5">
              <section className="grid gap-2">
                <label className="text-[13px] font-semibold text-label" htmlFor="monitor">
                  Monitor
                </label>
                <MonitorSelect
                  monitors={monitors}
                  value={selectedMonitor}
                  onChange={(deviceName) => {
                    setSelectedMonitor(deviceName);
                    setMonitor(deviceName).catch(console.error);
                  }}
                />
              </section>

              <section className="grid gap-3">
                <p className="m-0 text-[11px] font-bold uppercase tracking-[.15em] text-section">
                  Timing
                </p>
                <SliderControl
                  label="Poll Interval"
                  field="poll_interval_ms"
                  min="10"
                  max="1000"
                  step="10"
                  unit="ms"
                  description="How often the screen is sampled."
                  value={lampTuning.poll_interval_ms}
                  onChange={(v) => setLampTuning((c) => ({ ...c, poll_interval_ms: v }))}
                />
              </section>
              <section className="grid gap-4">
                <p className="m-0 text-[11px] font-bold uppercase tracking-[.15em] text-section">
                  Color sensitivity
                </p>
                <SliderControl
                  label="Gamma Correction"
                  field="gamma"
                  min="0.1"
                  max="5.0"
                  step="0.1"
                  unit="x"
                  description="Controls the brightness curve."
                  value={lampTuning.gamma}
                  onChange={(v) => setLampTuning((c) => ({ ...c, gamma: v }))}
                />
                <SliderControl
                  label="Hue Threshold"
                  field="hue_threshold"
                  min="0"
                  max="180"
                  unit="°"
                  description="Minimum hue shift before an update."
                  value={lampTuning.hue_threshold}
                  onChange={(v) => setLampTuning((c) => ({ ...c, hue_threshold: v }))}
                />
                <SliderControl
                  label="Saturation Threshold"
                  field="sat_threshold"
                  min="0"
                  max="1000"
                  step="10"
                  description="Minimum saturation shift before an update."
                  value={lampTuning.sat_threshold}
                  onChange={(v) => setLampTuning((c) => ({ ...c, sat_threshold: v }))}
                />
                <SliderControl
                  label="Brightness (V) Threshold"
                  field="v_threshold"
                  min="0"
                  max="1000"
                  step="10"
                  description="Minimum brightness shift before an update."
                  value={lampTuning.v_threshold}
                  onChange={(v) => setLampTuning((c) => ({ ...c, v_threshold: v }))}
                />
              </section>

              <Button type="submit" variant="primary">
                Save tuning <span aria-hidden="true">→</span>
              </Button>
            </div>
          </Panel>
        </form>
      </div>
    </main>
  </div>
);
}

export default App;
