import { listen, type UnlistenFn } from '@tauri-apps/api/event';

async function startConnectionListen(onValue: (event: boolean) => void): Promise<UnlistenFn> {
    return await listen<boolean>('lamp-connection', (event) => {
        onValue(event.payload);
    });
}

export { startConnectionListen }