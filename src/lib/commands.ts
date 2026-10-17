import { invoke } from "@tauri-apps/api/core";

import type { CaptureRegion } from "./bindings/CaptureRegion";
import type { Config } from "./bindings/Config";
import type { LampDeviceInfo } from "./bindings/LampDeviceInfo";
import type { LampTuning } from "./bindings/LampTuning";
import type { MonitorInfo } from "./bindings/MonitorInfo";

async function invokeConnectLamp(device: LampDeviceInfo) {
    await invoke("connect_lamp", { lamp: device }).catch((e) => {
        console.error(e);
    });
}

async function getDeviceInfo(): Promise<LampDeviceInfo> {
    return await invoke("get_device_info");
}

async function getConfig(): Promise<Config> {
    return await invoke("get_config");
}

async function invokeUpdateLampTuning(lampTuning: LampTuning) {
    await invoke("update_lamp_tuning", { lampTuning });
}

async function invokeUpdateCaptureRegions(captureRegions: CaptureRegion[]) {
    await invoke("update_capture_regions", { captureRegions });
}

async function getMonitors(): Promise<MonitorInfo[]> {
    return await invoke("get_monitors");
}

async function setMonitor(deviceName: string): Promise<void> {
    await invoke("set_monitor", { deviceName });
}

export {
    getConfig,
    getDeviceInfo,
    getMonitors,
    invokeConnectLamp,
    invokeUpdateCaptureRegions,
    invokeUpdateLampTuning,
    setMonitor,
};
export type { CaptureRegion, Config as LampConfig, LampDeviceInfo, LampTuning, MonitorInfo };
