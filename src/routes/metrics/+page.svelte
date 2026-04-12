<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '$lib/tauri';
  import { i18n } from '$lib/stores/i18n.svelte';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { dockerService } from '$lib/services/docker.service';
  import type { ContainerStats, SystemMetrics, StatsEvent } from '$lib/types';

  import { Button } from "$lib/components/ui/button";
  import * as Card from "$lib/components/ui/card";
  import * as Table from "$lib/components/ui/table";
  import { Badge } from "$lib/components/ui/badge";
  import { Progress } from "$lib/components/ui/progress"; // Note: Need to add progress component
  import {
    Activity,
    Cpu,
    MemoryStick,
    ArrowUpRight,
    ArrowDownLeft,
    Box,
    RefreshCw
  } from "lucide-svelte";
  import { cn } from '$lib/utils';

  let systemMetrics = $state<SystemMetrics | null>(null);
  let containerStats = $state<ContainerStats[]>([]);
  let isLoading = $state(true);
  let unlisten: (() => void) | null = null;

  async function loadMetrics() {
    if (!dockerStore.selectedEngine) return;
    try {
      const system = await dockerService.getSystemMetrics();
      systemMetrics = system;

      const containers = await dockerService.getContainerStats(dockerStore.selectedEngine);
      containerStats = containers;
    } catch (e) {
      console.error('Failed to load metrics', e);
    } finally {
      isLoading = false;
    }
  }

  async function startStreaming() {
    if (!dockerStore.selectedEngine) return;
    try {
      unlisten = await listen<StatsEvent>('docker-stats', (event) => {
        containerStats = event.payload.stats;
      });
      await dockerService.streamStats(dockerStore.selectedEngine);
    } catch (e) {
      console.error('Failed to start streaming metrics', e);
    }
  }

  onMount(() => {
    loadMetrics();
    startStreaming();

    const interval = setInterval(loadMetrics, 5000);
    return () => {
      clearInterval(interval);
      if (unlisten) unlisten();
      if (dockerStore.selectedEngine) {
        dockerService.stopStatsStream(dockerStore.selectedEngine);
      }
    };
  });
</script>

<div class="h-full flex flex-col bg-background">
  <!-- Header -->
  <header class="border-b bg-card/50 backdrop-blur-sm sticky top-0 z-10">
    <div class="container flex flex-col md:flex-row md:items-center justify-between py-4 gap-4">
      <div class="flex items-center gap-3">
        <div class="p-2 bg-primary/10 rounded-lg">
          <Activity class="h-6 w-6 text-primary" />
        </div>
        <div>
          <h1 class="text-2xl font-bold tracking-tight">{i18n.t('Metrics')}</h1>
          <p class="text-xs text-muted-foreground flex items-center gap-2">
            Real-time engine and container performance
          </p>
        </div>
      </div>
      <Button variant="outline" size="sm" onclick={loadMetrics} disabled={isLoading}>
        <RefreshCw class={cn("h-4 w-4 mr-2", isLoading && "animate-spin")} />
        {i18n.t('Refresh')}
      </Button>
    </div>
  </header>

  <div class="flex-1 overflow-auto">
    <div class="container py-6 space-y-8">
      <!-- System Overview -->
      <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card.Root>
          <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
            <Card.Title class="text-sm font-medium">CPU Usage</Card.Title>
            <Cpu class="h-4 w-4 text-muted-foreground" />
          </Card.Header>
          <Card.Content>
            <div class="text-2xl font-bold">
              {systemMetrics ? (systemMetrics.cpu_usage.reduce((a, b) => a + b, 0) / systemMetrics.cpu_usage.length).toFixed(1) : '0'}%
            </div>
            <p class="text-xs text-muted-foreground mt-1">
              Overall system CPU load
            </p>
          </Card.Content>
        </Card.Root>

        <Card.Root>
          <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
            <Card.Title class="text-sm font-medium">Memory Usage</Card.Title>
            <MemoryStick class="h-4 w-4 text-muted-foreground" />
          </Card.Header>
          <Card.Content>
            <div class="text-2xl font-bold">
              {systemMetrics ? systemMetrics.mem_percent.toFixed(1) : '0'}%
            </div>
            <div class="text-xs text-muted-foreground mt-1">
              {systemMetrics ? (systemMetrics.mem_used / 1024 / 1024 / 1024).toFixed(1) : '0'} GB /
              {systemMetrics ? (systemMetrics.mem_total / 1024 / 1024 / 1024).toFixed(1) : '0'} GB
            </div>
          </Card.Content>
        </Card.Root>

        <Card.Root>
          <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
            <Card.Title class="text-sm font-medium">Containers</Card.Title>
            <Box class="h-4 w-4 text-muted-foreground" />
          </Card.Header>
          <Card.Content>
            <div class="text-2xl font-bold">{containerStats.length}</div>
            <p class="text-xs text-muted-foreground mt-1">
              Active containers monitored
            </p>
          </Card.Content>
        </Card.Root>
      </div>

      <!-- Container Stats Table -->
      <div class="space-y-4">
        <div class="flex items-center gap-2 px-1">
          <Activity class="h-4 w-4 text-muted-foreground" />
          <h2 class="text-sm font-semibold uppercase tracking-wider text-muted-foreground">
            {i18n.t('ContainerStats') || 'Container Performance'}
          </h2>
        </div>

        <div class="rounded-md border bg-card overflow-hidden">
          <Table.Root>
            <Table.Header>
              <Table.Row>
                <Table.Head class="w-[25%]">{i18n.t('Name')}</Table.Head>
                <Table.Head class="w-[15%]">CPU %</Table.Head>
                <Table.Head class="w-[20%]">MEM Usage / Limit</Table.Head>
                <Table.Head class="w-[10%]">MEM %</Table.Head>
                <Table.Head class="w-[15%]">NET I/O</Table.Head>
                <Table.Head class="w-[15%]">BLOCK I/O</Table.Head>
              </Table.Row>
            </Table.Header>
            <Table.Body>
              {#each containerStats as stat (stat.id)}
                <Table.Row>
                  <Table.Cell>
                    <div class="font-bold truncate">{stat.name}</div>
                    <code class="text-[10px] text-muted-foreground font-mono">{stat.id.slice(0, 12)}</code>
                  </Table.Cell>
                  <Table.Cell>
                    <div class="flex items-center gap-2">
                      <span class="text-xs font-medium w-10">{stat.cpu_percent}</span>
                    </div>
                  </Table.Cell>
                  <Table.Cell class="text-xs">
                    {stat.mem_usage} / {stat.mem_limit}
                  </Table.Cell>
                  <Table.Cell>
                    <Badge variant="outline" class="text-[10px]">{stat.mem_percent}</Badge>
                  </Table.Cell>
                  <Table.Cell>
                    <div class="flex flex-col gap-0.5">
                      <span class="text-[10px] flex items-center gap-1"><ArrowDownLeft class="h-2 w-2 text-blue-500" /> {stat.net_io.split(' / ')[0]}</span>
                      <span class="text-[10px] flex items-center gap-1"><ArrowUpRight class="h-2 w-2 text-amber-500" /> {stat.net_io.split(' / ')[1]}</span>
                    </div>
                  </Table.Cell>
                  <Table.Cell>
                    <span class="text-[10px] font-mono text-muted-foreground">{stat.block_io}</span>
                  </Table.Cell>
                </Table.Row>
              {/each}
              {#if containerStats.length === 0}
                <Table.Row>
                  <Table.Cell colspan={6} class="h-32 text-center text-muted-foreground">
                    {#if isLoading}
                      <RefreshCw class="h-6 w-6 animate-spin mx-auto mb-2 opacity-20" />
                      Loading stats...
                    {:else}
                      No active containers to monitor.
                    {/if}
                  </Table.Cell>
                </Table.Row>
              {/if}
            </Table.Body>
          </Table.Root>
        </div>
      </div>
    </div>
  </div>
</div>
