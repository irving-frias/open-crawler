import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export type UpdaterStatus = 'idle' | 'checking' | 'available' | 'installing' | 'restart';

export interface UpdaterState {
  status: UpdaterStatus;
  version: string | null;
  error: string | null;
}

export async function checkForUpdates(): Promise<UpdaterState> {
  try {
    const update = await check();
    if (!update) return { status: 'idle', version: null, error: null };
    return { status: 'available', version: update.version, error: null };
  } catch (e) {
    return { status: 'idle', version: null, error: e instanceof Error ? e.message : String(e) };
  }
}

export async function downloadAndInstall(onProgress?: (percent: number) => void): Promise<void> {
  const update = await check();
  if (!update) return;
  let downloaded = 0;
  let total = 0;
  await update.downloadAndInstall((event) => {
    if (event.event === 'Started' && event.data.contentLength) {
      total = event.data.contentLength;
    } else if (event.event === 'Progress') {
      downloaded += event.data.chunkLength;
      onProgress?.(total > 0 ? Math.min(100, Math.round((downloaded / total) * 100)) : 0);
    }
  });
}

export async function relaunchApp(): Promise<void> {
  await relaunch();
}
