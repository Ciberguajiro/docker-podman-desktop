import { invoke as tauriInvoke } from '@tauri-apps/api/core';
import { listen as tauriListen, type UnlistenFn } from '@tauri-apps/api/event';

export async function invoke<T>(cmd: string, args: Record<string, any> = {}): Promise<T> {
  // @ts-ignore
  if (typeof window === 'undefined' || !window.__TAURI_INTERNALS__) {
    console.warn(`Tauri not detected. Mocking response for ${cmd}`);
    if (cmd === 'check_engine_cli_command') return true as any;
    if (cmd === 'docker_is_running') return {
      success: (window as any).__MOCK_DOCKER_RUNNING__ ?? false,
      output: '',
      error: (window as any).__MOCK_DOCKER_ERROR__ ?? null
    } as any;
    if (cmd === 'docker_info') return {
      containers: 0,
      containers_running: 0,
      containers_stopped: 0,
      images: 0,
      server_version: 'mock',
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
    } as any;
    if (cmd === 'docker_images') return ((window as any).__MOCK_IMAGES__ || []) as any;
    if (cmd === 'docker_ps') return ((window as any).__MOCK_CONTAINERS__ || []) as any;
    if (cmd === 'docker_logs') return `MOCK LOGS for ${args.containerId}\nLine 1\nLine 2\nTail was ${args.tail}` as any;
    if (cmd === 'docker_volumes') return [
      { name: 'vol1', driver: 'local', mountpoint: '/var/lib/docker/volumes/vol1/_data', created: '2023-01-01' },
      { name: 'vol2', driver: 'local', mountpoint: '/var/lib/docker/volumes/vol2/_data', created: '2023-01-02' }
    ] as any;
    if (cmd === 'docker_networks') return [
      { id: 'net1', name: 'bridge', driver: 'bridge', scope: 'local', subnet: '172.17.0.0/16' },
      { id: 'net2', name: 'host', driver: 'host', scope: 'local', subnet: '' }
    ] as any;
    if (cmd === 'docker_logs_stream') return { success: true } as any;
    if (cmd === 'docker_stop_logs_stream') return {} as any;
    return {} as any;
  }
  try {
    return await tauriInvoke<T>(cmd, args);
  } catch (e) {
    console.error(`Error invoking Tauri command ${cmd}:`, e);
    throw e;
  }
}

export async function listen<T>(event: string, handler: (event: { payload: T }) => void): Promise<UnlistenFn> {
  // @ts-ignore
  if (typeof window === 'undefined' || !window.__TAURI_INTERNALS__) {
    console.warn(`Tauri not detected. Mocking listener for ${event}`);
    return () => {};
  }
  return await tauriListen<T>(event, (e) => handler(e));
}
