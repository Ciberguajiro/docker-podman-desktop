import { invoke } from '../tauri';
import type {
  DockerInfo,
  DockerContainer,
  DockerImage,
  DockerVolume,
  DockerNetwork,
  CommandResult,
  ContainerFile,
  ImageHistoryEntry,
  Engine
} from '../types';

export const dockerService = {
  // Engine and Info
  async checkEngineCli(engine: Engine): Promise<boolean> {
    return invoke<boolean>('check_engine_cli_command', { engine });
  },

  async isRunning(engine: Engine): Promise<{ success: boolean; error?: string }> {
    return invoke<{ success: boolean; error?: string }>('docker_is_running', { engine });
  },

  async getInfo(engine: Engine): Promise<DockerInfo> {
    return invoke<DockerInfo>('docker_info', { engine });
  },

  async listenEvents(engine: Engine): Promise<void> {
    return invoke('docker_listen_events', { engine });
  },

  // Containers
  async getContainers(engine: Engine, all: boolean = true): Promise<DockerContainer[]> {
    return invoke<DockerContainer[]>('docker_ps', { engine, all });
  },

  async startContainer(engine: Engine, containerId: string): Promise<CommandResult> {
    return invoke<CommandResult>('docker_start', { engine, containerId });
  },

  async stopContainer(engine: Engine, containerId: string): Promise<CommandResult> {
    return invoke<CommandResult>('docker_stop', { engine, containerId });
  },

  async removeContainer(engine: Engine, containerId: string, force: boolean = true): Promise<CommandResult> {
    return invoke<CommandResult>('docker_remove_container', { engine, containerId, force });
  },

  async createContainer(engine: Engine, options: any): Promise<CommandResult> {
    return invoke<CommandResult>('docker_create_container', { engine, ...options });
  },

  async exportContainer(engine: Engine, containerId: string, path: string): Promise<CommandResult> {
    return invoke<CommandResult>('docker_export_container', { engine, containerId, path });
  },

  async inspect(engine: Engine, id: string): Promise<string> {
    return invoke<string>('docker_inspect', { engine, id });
  },

  // Container Files & Exec
  async listContainerFiles(engine: Engine, containerId: string, path: string): Promise<ContainerFile[]> {
    return invoke<ContainerFile[]>('docker_container_ls', { engine, containerId, path });
  },

  async readContainerFile(engine: Engine, containerId: string, path: string): Promise<string> {
    return invoke<string>('docker_container_read_file', { engine, containerId, path });
  },

  async writeContainerFile(engine: Engine, containerId: string, path: string, content: string): Promise<CommandResult> {
    return invoke<CommandResult>('docker_container_write_file', { engine, containerId, path, content });
  },

  async exec(engine: Engine, options: any): Promise<CommandResult> {
    return invoke<CommandResult>('docker_exec', { engine, ...options });
  },

  // Logs
  async streamLogs(engine: Engine, containerId: string, tail: string): Promise<CommandResult> {
    return invoke<CommandResult>('docker_logs_stream', { engine, containerId, tail });
  },

  async stopLogsStream(): Promise<void> {
    return invoke('docker_stop_logs_stream');
  },

  // Images
  async getImages(engine: Engine): Promise<DockerImage[]> {
    return invoke<DockerImage[]>('docker_images', { engine });
  },

  async getImageHistory(engine: Engine, imageId: string): Promise<ImageHistoryEntry[]> {
    return invoke<ImageHistoryEntry[]>('docker_image_history', { engine, imageId });
  },

  async removeImage(engine: Engine, imageId: string, force: boolean = false): Promise<CommandResult> {
    return invoke<CommandResult>('docker_remove_image', { engine, imageId, force });
  },

  async pullImage(engine: Engine, image: string): Promise<CommandResult> {
    return invoke<CommandResult>('docker_pull', { engine, image });
  },

  async stopPull(): Promise<void> {
    return invoke('docker_stop_pull');
  },

  async tagImage(engine: Engine, options: any): Promise<CommandResult> {
    return invoke<CommandResult>('docker_tag_image', { engine, ...options });
  },

  async buildImage(engine: Engine, options: any): Promise<CommandResult> {
    return invoke<CommandResult>('docker_build_image', { engine, ...options });
  },

  async stopBuild(): Promise<void> {
    return invoke('docker_stop_build');
  },

  async importImage(engine: Engine, options: any): Promise<CommandResult> {
    return invoke<CommandResult>('docker_import_image', { engine, ...options });
  },

  // Volumes
  async getVolumes(engine: Engine): Promise<DockerVolume[]> {
    return invoke<DockerVolume[]>('docker_volumes', { engine });
  },

  async createVolume(engine: Engine, name: string): Promise<CommandResult> {
    return invoke<CommandResult>('docker_create_volume', { engine, name });
  },

  async removeVolume(engine: Engine, name: string): Promise<CommandResult> {
    return invoke<CommandResult>('docker_remove_volume', { engine, name });
  },

  // Networks
  async getNetworks(engine: Engine): Promise<DockerNetwork[]> {
    return invoke<DockerNetwork[]>('docker_networks', { engine });
  },

  async createNetwork(engine: Engine, name: string, driver: string): Promise<CommandResult> {
    return invoke<CommandResult>('docker_create_network', { engine, name, driver });
  },

  async removeNetwork(engine: Engine, id: string): Promise<CommandResult> {
    return invoke<CommandResult>('docker_remove_network', { engine, id });
  },

  // Prune
  async prune(engine: Engine, type_: string): Promise<CommandResult> {
    return invoke<CommandResult>('docker_prune', { engine, type_ });
  },

  // Compose
  async composeUp(engine: Engine, projectName: string): Promise<CommandResult> {
    return invoke<CommandResult>('docker_compose_up', { engine, projectName });
  },

  async composeDown(engine: Engine, projectName: string): Promise<CommandResult> {
    return invoke<CommandResult>('docker_compose_down', { engine, projectName });
  },

  async composeRestart(engine: Engine, projectName: string): Promise<CommandResult> {
    return invoke<CommandResult>('docker_compose_restart', { engine, projectName });
  },

  // Marketplace
  async searchHub(engine: Engine, query: string): Promise<any> {
    return invoke<any>('docker_hub_search', { engine, query });
  },

  // Metrics
  async getSystemMetrics(): Promise<any> {
    return invoke<any>('get_system_metrics');
  },

  async getContainerStats(engine: Engine): Promise<any[]> {
    return invoke<any[]>('docker_container_stats', { engine });
  },

  async streamStats(engine: Engine): Promise<void> {
    return invoke('docker_stats_stream', { engine });
  },

  async stopStatsStream(engine: Engine): Promise<void> {
    return invoke('docker_stop_stats_stream', { engine });
  },

  async runElevatedCommand(command: string, args: string[], elevated: boolean): Promise<CommandResult> {
    return invoke<CommandResult>('run_elevated_command', { command, args, elevated });
  }
};
