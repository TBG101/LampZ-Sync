import { useEffect, useState, type ChangeEvent } from "react";
import "./App.css";
import Button from "./components/Button";
import Input from "./components/Input";
import {
  getConfig,
  invokeConnectLamp,
  invokeUpdateLampTuning,
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

function SliderControl({
  label,
  field,
  min,
  max,
  step = "1",
  unit = "",
  description,
  value,
  onChange,
}: {
  label: string;
  field: keyof LampTuning;
  min: string;
  max: string;
  step?: string;
  unit?: string;
  description: string;
  value: number;
  onChange: (value: number) => void;
}) {
  return (
    <div className="tuning-control">
      <div className="control-heading">
        <div>
          <label className="control-label" htmlFor={field}>{label}</label>
          <p className="control-description">{description}</p>
        </div>
        <output className="control-value" htmlFor={field}>
          {value} <span>{unit}</span>
        </output>
      </div>
      <input
        id={field}
        type="range"
        name={field}
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
        className="range-input"
      />
      <div className="range-scale" aria-hidden="true"><span>{min}{unit}</span><span>{max}{unit}</span></div>
    </div>
  );
}

function App() {
  const [lamp, setLamp] = useState<LampForm>({
    id: "",
    ip: "",
    localKey: "",
  });
  const [lampConnection, setLampConnection] = useState(false);
  const [lampTuning, setLampTuning] = useState<LampTuning>(defaultLampTuning);

  useEffect(() => {
    getConfig().catch(console.error).then(config => {
      if (config) {
        setLamp({
          id: config.lamp_connection.lamp_id ?? "",
          ip: config.lamp_connection.lamp_ip ?? "",
          localKey: config.lamp_connection.lamp_key ?? "",
        });
        setLampTuning(config.lamp_tuning);
      }
    });

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
    <main className="app-shell">
      <header className="app-header">
        <div>
          <p className="eyebrow">LampZ Sync / Control center</p>
        </div>
        <div className={`connection-status ${lampConnection ? "is-connected" : ""}`}>
          <span className="status-dot" />
          <span>{lampConnection ? "Connected" : "Not connected"}</span>
        </div>
      </header>

      <div className="dashboard-grid">
        <form
          className="panel connection-panel"
          onSubmit={(e) => {
            e.preventDefault();
            invokeConnectLamp({
              lampId: lamp.id,
              lampIp: lamp.ip,
              lampKey: lamp.localKey,
            });
          }}
        >
          <div className="panel-heading">
            <span className="panel-index">01</span>
            <div><h2>Device connection</h2><p>Connect to your lamp over the local network.</p></div>
          </div>

          <div className="field-stack">
            <div className="field-group">
              <label htmlFor="id">Lamp ID</label>
              <Input
                name="id"
                value={lamp.id}
                onChange={(event: ChangeEvent<HTMLInputElement>) => updateField("id", event)}
                placeholder="e.g. lamp_12345"
              />
            </div>

            <div className="field-group">
              <label htmlFor="ip">IP address</label>
              <Input
                name="ip"
                value={lamp.ip}
                onChange={(event: ChangeEvent<HTMLInputElement>) => updateField("ip", event)}
                placeholder="192.168.1.100"
              />
            </div>

            <div className="field-group">
              <label htmlFor="localKey">Local key</label>
              <Input
                name="localKey"
                value={lamp.localKey}
                onChange={(event: ChangeEvent<HTMLInputElement>) => updateField("localKey", event)}
                placeholder="Secret key"
              />
            </div>
          </div>

          <Button type="submit" className="primary-action">Connect lamp <span aria-hidden="true">→</span></Button>
        </form>

        <form
          className="panel tuning-panel"
          onSubmit={(event) => {
            event.preventDefault();
            invokeUpdateLampTuning(lampTuning).catch(console.error);
          }}
        >
          <div className="panel-heading">
            <span className="panel-index">02</span>
            <div><h2>Visual response</h2><p>Shape how quickly color changes reach the lamp.</p></div>
          </div>

          <div className="tuning-section">
            <p className="section-label">Timing</p>
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
            {/* <SliderControl
              label="Mailbox Timeout"
              field="mailbox_timeout_secs"
              min="1"
              max="30"
              unit="sec"
              description="Maximum wait for a lamp response."
              value={lampTuning.mailbox_timeout_secs}
              onChange={(v) => setLampTuning((c) => ({ ...c, mailbox_timeout_secs: v }))}
            /> */}
          </div>
          <div className="tuning-section">
            <p className="section-label">Color sensitivity</p>
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
          </div>

          <Button type="submit" className="primary-action">Save tuning <span aria-hidden="true">→</span></Button>
        </form>
      </div>
    </main>
  );
}

export default App;