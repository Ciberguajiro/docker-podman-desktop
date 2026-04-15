import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { toastStore } from '../stores/toasts.svelte';

export const updaterService = {
  async checkForUpdates(silent: boolean = true) {
    // @ts-ignore
    if (typeof window === 'undefined' || !window.__TAURI_INTERNALS__) {
        console.warn('Updater mocked: No Tauri internals');
        if (!silent) toastStore.info('Updater mocked: No Tauri internals');
        return null;
    }

    try {
      const update = await check();
      if (update) {
        toastStore.info(`New version available: ${update.version}`);

        console.log(`Update found: ${update.version}`);

        // await update.downloadAndInstall();
        // await relaunch();
        return update;
      } else if (!silent) {
        toastStore.info('Your application is up to date.');
      }
    } catch (error) {
      console.error('Failed to check for updates:', error);
      if (!silent) {
        toastStore.error('Failed to check for updates');
      }
    }
    return null;
  }
};
