import type { DockerInfo, DockerEvent, Engine } from '../types';
import { invoke, listen } from '../tauri';

function createDockerStore() {
  let selectedEngine = $state<Engine | null>(null);
  let isRunning = $state(false);
  let isCliInstalled = $state(true);
  let dockerError = $state<string | null>(null);
  let listeningEngine: Engine | null = null;
  let refreshCounter = $state(0);
  let info = $state<DockerInfo>({
    containers: 0,
    containers_running: 0,
    containers_stopped: 0,
    images: 0,
    server_version: '',
    storage_driver: '',
    operating_system: '',
    architecture: '',
    cpus: 0,
    memory: '',
    kernel_version: '',
    os_type: '',
    cgroup_driver: '',
    cgroup_version: '',
    logging_driver: '',
    docker_root_dir: '',
  });

  async function checkStatus() {
    if (!selectedEngine) {
      isRunning = false;
      isCliInstalled = true;
      return;
    }

    try {
      isCliInstalled = await invoke<boolean>('check_engine_cli_command', { engine: selectedEngine });
      if (isCliInstalled) {
        const res = await invoke<{ success: boolean; error?: string }>('docker_is_running', { engine: selectedEngine });
        isRunning = res.success;
        dockerError = res.error || null;
        if (isRunning) {
          info = await invoke<DockerInfo>('docker_info', { engine: selectedEngine });
          if (listeningEngine !== selectedEngine) {
            listenToEvents();
          }
        }
      } else {
        isRunning = false;
        dockerError = null;
      }
    } catch (e) {
      console.error('Failed to check docker status', e);
      isRunning = false;
    }
  }

  async function listenToEvents() {
    if (!selectedEngine || listeningEngine === selectedEngine) return;
    const engineToListen = selectedEngine;

    try {
      if (listeningEngine === null) {
        await listen<DockerEvent>('docker-event', (event) => {
          console.log('Docker event received:', event.payload.Action, event.payload.Type);
          refreshCounter++;
        });
      }
      listeningEngine = engineToListen;
      await invoke('docker_listen_events', { engine: engineToListen });
    } catch (e) {
      console.error('Failed to listen to docker events', e);
      listeningEngine = null;
    }
  }

  return {
    get selectedEngine() { return selectedEngine; },
    set selectedEngine(value) {
      selectedEngine = value;
      this.checkStatus();
    },
    invoke<T>(cmd: string, args: Record<string, any> = {}): Promise<T> {
      if (!selectedEngine && (cmd.startsWith('docker_') || cmd === 'check_engine_cli_command')) {
         return Promise.reject("No engine selected");
      }
      return invoke<T>(cmd, { engine: selectedEngine, ...args });
    },
    get isRunning() { return isRunning; },
    get isCliInstalled() { return isCliInstalled; },
    get dockerError() { return dockerError; },
    get info() { return info; },
    get refreshCounter() { return refreshCounter; },
    set refreshCounter(value) { refreshCounter = value; },
    triggerRefresh() { refreshCounter++; },
    checkStatus,
  };
}

export const dockerStore = createDockerStore();
