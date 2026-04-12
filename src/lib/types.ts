export interface DockerContainer {
  id: string;
  name: string;
  image: string;
  status: string;
  state: string;
  created: string;
  ports: string[];
  compose_project?: string;
  compose_service?: string;
}

export interface ContainerFile {
  name: string;
  is_dir: boolean;
  size: string;
  modified: string;
  permissions: string;
}

export interface DockerImage {
  id: string;
  repository: string;
  tag: string;
  size: string;
  created: string;
}

export interface ImageHistoryEntry {
  id: string;
  created: string;
  created_by: string;
  size: string;
  comment: string;
}

export interface DockerVolume {
  name: string;
  driver: string;
  mountpoint: string;
  created: string;
}

export interface DockerNetwork {
  id: string;
  name: string;
  driver: string;
  scope: string;
  subnet: string;
}

export interface DockerInfo {
  containers: number;
  containers_running: number;
  containers_stopped: number;
  images: number;
  server_version: string;
  storage_driver: string;
  operating_system: string;
  architecture: string;
  cpus: number;
  memory: string;
  kernel_version: string;
  os_type: string;
  cgroup_driver: string;
  cgroup_version: string;
  logging_driver: string;
  docker_root_dir: string;
}

export interface ContainerStats {
  id: string;
  name: string;
  cpu_percent: string;
  mem_usage: string;
  mem_limit: string;
  mem_percent: string;
  net_io: string;
  block_io: string;
  pids: string;
}

export interface SystemMetrics {
  cpu_usage: number[];
  mem_total: number;
  mem_used: number;
  mem_free: number;
  mem_percent: number;
}

export interface AllMetrics {
  system: SystemMetrics;
  containers: ContainerStats[];
}

export interface CommandResult {
  success: boolean;
  output: string;
  error?: string;
}

export type Engine = 'docker' | 'podman';

export interface PullEvent {
  status: string;
  progress?: string;
  is_error: boolean;
}

export interface BuildEvent {
  line: string;
  is_error: boolean;
}

export interface StatsEvent {
  stats: ContainerStats[];
}

export interface LogEvent {
  line: string;
  is_error: boolean;
}

export interface DockerEvent {
  status: string;
  id: string;
  from: string;
  Type: string;
  Action: string;
  Actor: DockerEventActor;
  time: number;
}

export interface DockerEventActor {
  ID: string;
  Attributes: any;
}

export type ToastType = 'Success' | 'Error' | 'Info' | 'Warning';

export interface ToastMessage {
  id: number;
  message: string;
  toast_type: ToastType;
}

export interface AppSettings {
  theme: string;
  language: string;
  autoRefresh: boolean;
  refreshInterval: number;
}

export enum View {
  Containers = 'Containers',
  Images = 'Images',
  Volumes = 'Volumes',
  Networks = 'Networks',
  Metrics = 'Metrics',
  Marketplace = 'Marketplace',
  Settings = 'Settings',
}
