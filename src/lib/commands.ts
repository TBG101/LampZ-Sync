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

type LampConfig = {
    lamp_connection: {
        lamp_id: string | null;
        lamp_ip: string | null;
        lamp_key: string | null;
    };
    lamp_tuning: LampTuning;
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


export { invokeConnectLamp, getConfig, getDeviceInfo, invokeUpdateLampTuning };
export type { LampConfig, LampDeviceInfo, LampTuning };
