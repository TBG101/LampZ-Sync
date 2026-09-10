import { invoke } from "@tauri-apps/api/core";

type LampDeviceInfo = {
    lampId: string;
    lampIp: string;
    lampKey: string;
};

type LampTuning = {
    poll_interval_ms: number;
    mailbox_timeout_secs: number;
    gamma: number;
    hue_threshold: number;
    sat_threshold: number;
    v_threshold: number;
};

type CaptureRegion = {
    left: number;
    right: number;
    top: number;
    bottom: number;
};

type LampConfig = {
    lamp_connection: {
        lamp_id: string | null;
        lamp_ip: string | null;
        lamp_key: string | null;
    };
    lamp_tuning: LampTuning;
    monitor_device_name: string | null;
    capture_regions: CaptureRegion[];
};

type MonitorInfo = {
    device_name: string;
    name: string;
    width: number;
    height: number;
    refresh_rate: number;
};

async function invokeConnectLamp(device: LampDeviceInfo) {
    await invoke("connect_lamp", { lamp: device }).catch((e) => {
        console.error(e);
    })
}

async function getDeviceInfo(): Promise<LampDeviceInfo> {
    return await invoke("get_device_info");
}

async function getConfig(): Promise<LampConfig> {
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
export type { CaptureRegion, LampConfig, LampDeviceInfo, LampTuning, MonitorInfo };
