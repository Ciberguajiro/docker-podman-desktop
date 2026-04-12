<script lang="ts">
  import { View, type DockerInfo } from '@lib/types';
  import { i18n } from '@lib/stores/i18n.svelte';
  import { dockerStore } from '@lib/stores/docker.svelte';
  import * as Select from "$lib/components/ui/select";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { cn } from "$lib/utils";
  import { page } from "$app/state";

  let { dockerRunning, dockerInfo } = $props<{
    dockerRunning: boolean;
    dockerInfo: DockerInfo;
  }>();

  const views = $derived([
    { urls: "/containers" , id: View.Containers, label: i18n.t('Containers'), icon: '📦' },
    { urls: "/images" , id: View.Images, label: i18n.t('Images'), icon: '🖼️' },
    { urls: "/volumes" , id: View.Volumes, label: i18n.t('Volumes'), icon: '💾' },
    { urls: "/networks" , id: View.Networks, label: i18n.t('Networks'), icon: '🌐' },
    { urls: "/metrics" , id: View.Metrics, label: i18n.t('Metrics'), icon: '📊' },
    { urls: "/marketplaces" , id: View.Marketplace, label: i18n.t('Marketplace'), icon: '🛒' },
    { urls: "/settings" , id: View.Settings, label: i18n.t('Settings'), icon: '⚙️' },
  ]);

  const engines = [
    { value: "docker", label: "Docker" },
    { value: "podman", label: "Podman" }
  ];

  let selectedEngineValue = $derived(dockerStore.selectedEngine || "");
</script>

<aside class="w-20 lg:w-64 bg-sidebar border-r border-sidebar-border flex flex-col h-full transition-all duration-300">
  <div class="p-4 flex flex-col gap-4">
    <div class="flex items-center gap-3">
      <div class="flex items-center justify-center w-10 h-10 rounded-full bg-primary text-primary-foreground shrink-0">
        <svg width="24" height="24" viewBox="0 0 50 50" fill="currentColor">
          <path d="M45 20c-1.5-1-3.5-1.5-5.5-1 0-2-1-4-2.5-5.5-1.5-1.5-3.5-2.5-5.5-3-2 0-4 .5-5.5 1.5-1-2-2.5-3.5-4.5-4.5-2-1-4-1.5-6-1-2 .5-4 1.5-5.5 3-1.5 1.5-2.5 3-3 5-.5 2-.5 4 0 6H2v5h3c0 3 1 6 3 8.5 2 2.5 4.5 4.5 7.5 5.5 3 1 6 1.5 9 1 3-.5 6-2 8-4 2-2 3.5-4.5 4-7.5h3v-5h-2c1-1.5 1.5-3 1.5-5 0-1.5-.5-3-1.5-4.5-.5 0-1.5-.5-2.5-.5zM15 30h-3v-5h3v5zm5 0h-3v-5h3v5zm5 0h-3v-5h3v5zm5 0h-3v-5h3v5zm5-5c0 1-.5 2-1 3h-2v-5h2c.5 1 1 1.5 1 2z"/>
        </svg>
      </div>
      <span class="text-xl font-bold tracking-tight hidden lg:inline text-sidebar-foreground">
        {dockerStore.selectedEngine === 'podman' ? 'Podman' : 'Docker'}
      </span>
    </div>

    <div class="w-full hidden lg:block">
      <Select.Root
        type="single"
        value={selectedEngineValue}
        onValueChange={(v) => { if (v) dockerStore.selectedEngine = v as any; }}
      >
        <Select.Trigger class="w-full h-9 font-bold bg-sidebar text-sidebar-foreground border-sidebar-border">
          {engines.find(e => e.value === selectedEngineValue)?.label || i18n.t('SelectEngine')}
        </Select.Trigger>
        <Select.Content>
          {#each engines as engine}
            <Select.Item value={engine.value}>{engine.label}</Select.Item>
          {/each}
        </Select.Content>
      </Select.Root>
    </div>

    <div class="lg:hidden flex justify-center">
      <Button
        variant="outline"
        size="icon"
        class="w-8 h-8 rounded-full border-sidebar-border text-sidebar-foreground"
        onclick={() => dockerStore.selectedEngine = dockerStore.selectedEngine === 'docker' ? 'podman' : 'docker'}
      >
        {dockerStore.selectedEngine === 'podman' ? 'P' : 'D'}
      </Button>
    </div>
  </div>

  <nav class="flex-1 px-3 py-4 overflow-y-auto">
    <div class="flex flex-col gap-1">
      {#each views as view}
        {@const isActive = page.url.pathname === view.urls}
        <a
          href={view.urls}
          class={cn(
            "flex items-center gap-3 px-3 py-2 rounded-md transition-colors duration-200",
            "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
            isActive ? "bg-sidebar-accent text-sidebar-accent-foreground font-medium" : "text-sidebar-foreground/70"
          )}
        >
          <span class="text-xl">{view.icon}</span>
          <span class="hidden lg:inline truncate">{view.label}</span>
          {#if view.id === View.Containers && dockerInfo.containers > 0}
            <Badge variant="secondary" class="ml-auto hidden lg:flex h-5 px-1.5 text-[10px] min-w-[20px] justify-center">
              {dockerInfo.containers}
            </Badge>
          {:else if view.id === View.Images && dockerInfo.images > 0}
            <Badge variant="secondary" class="ml-auto hidden lg:flex h-5 px-1.5 text-[10px] min-w-[20px] justify-center">
              {dockerInfo.images}
            </Badge>
          {/if}
        </a>
      {/each}
    </div>
  </nav>

  <div class="p-4 border-t border-sidebar-border mt-auto bg-sidebar">
    <div class="flex items-center gap-2 mb-1 justify-center lg:justify-start">
      <div class={cn("w-2 h-2 rounded-full", dockerRunning ? "bg-green-500" : "bg-red-500")}></div>
      <span class="text-[10px] lg:text-xs font-semibold uppercase tracking-wider text-sidebar-foreground/70 truncate">
        {dockerRunning ? i18n.t('EngineRunning') : i18n.t('EngineStopped')}
      </span>
    </div>
    {#if dockerRunning && dockerInfo.server_version}
      <div class="text-[10px] text-sidebar-foreground/50 font-mono text-center lg:text-left truncate">v{dockerInfo.server_version}</div>
    {/if}
  </div>
</aside>
