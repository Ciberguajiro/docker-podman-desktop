<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke, listen } from '$lib/tauri';
  import { i18n } from '$lib/stores/i18n.svelte';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import type { AllMetrics, StatsEvent, ContainerStats } from '$lib/types';

  let metrics = $state<AllMetrics | null>(null);
  let isLoading = $state(true);
  let lastRefreshed = $state<Date | null>(null);
  let unlistenStats: (() => void) | null = null;
  let containerStatsMap = $state<Record<string, ContainerStats>>({});

  async function loadMetrics() {
    isLoading = true;
    try {
      const system = await invoke<any>('get_system_metrics');
      if (Object.keys(containerStatsMap).length === 0) {
          const containers = dockerStore.selectedEngine ? await dockerStore.invoke<any[]>('docker_container_stats') : [];
          containers.forEach(c => { containerStatsMap[c.id] = c; });
      }
      metrics = { system, containers: Object.values(containerStatsMap) };
      lastRefreshed = new Date();
    } catch (e) {
      console.error('Failed to load metrics', e);
      toastStore.error(`${i18n.t('Metrics')}: ${e}`);
    } finally {
      isLoading = false;
    }
  }

  async function startStatsStream() {
    if (!dockerStore.selectedEngine) return;

    if (unlistenStats) unlistenStats();

    unlistenStats = await listen<StatsEvent>('stats-event', (event) => {
      event.payload.stats.forEach(s => {
        containerStatsMap[s.id] = s;
      });
      if (metrics) {
        metrics.containers = Object.values(containerStatsMap);
      }
    });

    try {
      await dockerStore.invoke('docker_stats_stream');
    } catch (e) {
      console.error('Failed to start stats stream', e);
    }
  }

  async function stopStatsStream() {
    if (unlistenStats) {
      unlistenStats();
      unlistenStats = null;
    }
    await dockerStore.invoke('docker_stop_stats_stream');
  }

  $effect(() => {
    if (dockerStore.refreshCounter >= 0) {
      loadMetrics();
    }
  });

  onMount(() => {
    loadMetrics();
    startStatsStream();
  });

  onDestroy(() => {
    stopStatsStream();
  });

  const system = $derived(metrics?.system);
  const containers = $derived(metrics?.containers || []);

  const cpuUsage = $derived(system?.cpu_usage || []);
  const avgCpu = $derived(cpuUsage.length ? (cpuUsage.reduce((a: number, b: number) => a + b, 0) / cpuUsage.length).toFixed(1) : '0');

  const memUsedGB = $derived(system ? (system.mem_used / 1024 / 1024 / 1024).toFixed(2) : '0');
  const memTotalGB = $derived(system ? (system.mem_total / 1024 / 1024 / 1024).toFixed(2) : '0');
</script>

<div class="p-6">
  <div class="flex justify-between items-center mb-6">
    <h1 class="text-3xl font-bold">{i18n.t('Metrics')}</h1>
    <div class="flex flex-col items-end">
      <button class="btn btn-outline btn-sm" onclick={loadMetrics} disabled={isLoading}>
        {#if isLoading}
          <span class="loading loading-spinner loading-xs"></span>
        {/if}
        🔄 {i18n.t('Refresh')}
      </button>
      {#if lastRefreshed}
        <span class="text-[10px] opacity-50 mt-1">{i18n.t('LastUpdated')}: {lastRefreshed.toLocaleTimeString()}</span>
      {/if}
    </div>
  </div>

  {#if isLoading && !system}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
      <div class="card bg-base-200 shadow-xl border border-base-300">
        <div class="card-body">
          <div class="skeleton h-6 w-1/3 mb-4"></div>
          <div class="skeleton h-12 w-1/2 mb-4"></div>
          <div class="flex flex-wrap gap-1">
            {#each Array(8) as _}
              <div class="skeleton w-8 h-2 rounded"></div>
            {/each}
          </div>
        </div>
      </div>
      <div class="card bg-base-200 shadow-xl border border-base-300">
        <div class="card-body">
          <div class="skeleton h-6 w-1/3 mb-4"></div>
          <div class="skeleton h-12 w-1/2 mb-4"></div>
          <div class="skeleton h-4 w-full"></div>
        </div>
      </div>
    </div>
    <div class="skeleton h-8 w-1/4 mb-4"></div>
    <div class="bg-base-200 rounded-lg shadow overflow-hidden">
      <div class="p-4 space-y-2">
        {#each Array(5) as _}
          <div class="skeleton h-10 w-full"></div>
        {/each}
      </div>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6 mb-8">
      <div class="card bg-base-200 shadow-xl border border-base-300">
        <div class="card-body">
          <h2 class="card-title text-primary">{i18n.t('CPUUsage')}</h2>
          <div class="text-5xl font-bold my-4">{avgCpu}%</div>
          <div class="flex flex-wrap gap-1">
            {#each cpuUsage as cpu, i}
              <div class="w-8 h-2 bg-base-300 rounded overflow-hidden">
                <div class="h-full bg-primary" style="width: {cpu}%"></div>
              </div>
            {/each}
          </div>
          <p class="text-sm opacity-50 mt-2">{cpuUsage.length} {i18n.t('Cores')}</p>
        </div>
      </div>

      <div class="card bg-base-200 shadow-xl border border-base-300">
        <div class="card-body">
          <h2 class="card-title text-secondary">{i18n.t('MemoryUsage')}</h2>
          <div class="text-5xl font-bold my-4">{system?.mem_percent?.toFixed(1) ?? '0'}%</div>
          <progress class="progress progress-secondary w-full" value={system?.mem_percent ?? 0} max="100"></progress>
          <p class="text-sm opacity-50 mt-2">{memUsedGB} GB / {memTotalGB} GB</p>
        </div>
      </div>
    </div>

    <h2 class="text-2xl font-bold mb-4">{i18n.t('ContainerMetrics')}</h2>
    <div class="overflow-x-auto bg-base-200 rounded-lg shadow">
      <table class="table table-zebra w-full">
        <thead>
          <tr>
            <th>{i18n.t('Container')}</th>
            <th>{i18n.t('CPUPerc')}</th>
            <th>{i18n.t('MemPerc')}</th>
            <th>{i18n.t('MemUsageLimit')}</th>
            <th>{i18n.t('NetIO')}</th>
            <th>{i18n.t('BlockIO')}</th>
            <th>{i18n.t('PIDs')}</th>
          </tr>
        </thead>
        <tbody>
          {#each containers as c (c.id)}
            <tr>
              <td>
                <div class="font-bold">{c.name}</div>
                <div class="text-xs opacity-50 font-mono">{c.id.slice(0, 12)}</div>
              </td>
              <td class="font-mono">{c.cpu_percent}</td>
              <td class="font-mono">{c.mem_percent}</td>
              <td class="font-mono">{c.mem_usage} / {c.mem_limit}</td>
              <td class="font-mono">{c.net_io}</td>
              <td class="font-mono">{c.block_io}</td>
              <td class="font-mono">{c.pids}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
