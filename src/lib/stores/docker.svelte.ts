import type { DockerInfo, DockerEvent, Engine } from '../types';
import { listen, invoke as tauriInvoke } from '../tauri';
import { dockerService } from '../services/docker.service';

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
      isCliInstalled = await dockerService.checkEngineCli(selectedEngine);
      if (isCliInstalled) {
        const res = await dockerService.isRunning(selectedEngine);
        isRunning = res.success;
        dockerError = res.error || null;
        if (isRunning) {
          info = await dockerService.getInfo(selectedEngine);
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
        await listen<DockerEvent>('docker-event', () => {
          refreshCounter++;
        });
      }
      listeningEngine = engineToListen;
      await dockerService.listenEvents(engineToListen);
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
    async invoke<T>(cmd: string, args: Record<string, any> = {}): Promise<T> {
      if (!selectedEngine && (cmd.startsWith('docker_') || cmd === 'check_engine_cli_command')) {
         return Promise.reject("No engine selected");
      }
      return tauriInvoke<T>(cmd, { engine: selectedEngine, ...args });
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
