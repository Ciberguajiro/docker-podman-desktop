<script lang="ts">
  import { View, type DockerInfo } from '@lib/types';
  import { i18n } from '@lib/stores/i18n.svelte';
  import { dockerStore } from '@lib/stores/docker.svelte';
  import * as Select from "$lib/components/ui/select";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import { cn } from "$lib/utils";
  import { page } from "$app/state";
  import {
    Box,
    Layers,
    Database,
    Network,
    Activity,
    ShoppingCart,
    Settings,
    Container as ContainerIcon,
    ChevronDown,
    Zap,
    CircleStop,
    Server,
    LayoutDashboard
  } from "lucide-svelte";

  let { dockerRunning, dockerInfo } = $props<{
    dockerRunning: boolean;
    dockerInfo: DockerInfo;
  }>();

  const views = $derived([
    { urls: "/" , id: View.Dashboard, label: i18n.t('Dashboard'), icon: LayoutDashboard },
    { urls: "/containers" , id: View.Containers, label: i18n.t('Containers'), icon: Box },
    { urls: "/images" , id: View.Images, label: i18n.t('Images'), icon: Layers },
    { urls: "/volumes" , id: View.Volumes, label: i18n.t('Volumes'), icon: Database },
    { urls: "/networks" , id: View.Networks, label: i18n.t('Networks'), icon: Network },
    { urls: "/metrics" , id: View.Metrics, label: i18n.t('Metrics'), icon: Activity },
    { urls: "/marketplaces" , id: View.Marketplace, label: i18n.t('Marketplace'), icon: ShoppingCart },
    { urls: "/settings" , id: View.Settings, label: i18n.t('Settings'), icon: Settings },
  ]);

  const engines = [
    { value: "docker", label: "Docker" },
    { value: "podman", label: "Podman" }
  ];

  let selectedEngineValue = $derived(dockerStore.selectedEngine || "");
</script>

<aside class="w-16 lg:w-64 bg-sidebar border-r border-sidebar-border flex flex-col h-full transition-all duration-300">
  <div class="p-4 flex flex-col gap-6">
    <div class="flex items-center gap-3">
      <div class="flex items-center justify-center w-8 h-8 rounded-lg bg-primary text-primary-foreground shrink-0 shadow-lg shadow-primary/20">
        <Server class="w-5 h-5" />
      </div>
      <span class="text-lg font-bold tracking-tight hidden lg:inline text-sidebar-foreground">
        {dockerStore.selectedEngine === 'podman' ? 'Podman' : 'Docker'}
      </span>
    </div>

    <div class="w-full hidden lg:block">
      <Select.Root
        type="single"
        value={selectedEngineValue}
        onValueChange={(v) => { if (v) dockerStore.selectedEngine = v as any; }}
      >
        <Select.Trigger class="w-full h-9 font-medium bg-sidebar text-sidebar-foreground border-sidebar-border hover:bg-sidebar-accent/50">
          <div class="flex items-center gap-2">
            <Zap class="w-3.5 h-3.5 text-primary" />
            {engines.find(e => e.value === selectedEngineValue)?.label || i18n.t('SelectEngine')}
          </div>
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
        class="w-8 h-8 rounded-lg border-sidebar-border text-sidebar-foreground hover:bg-sidebar-accent"
        onclick={() => dockerStore.selectedEngine = dockerStore.selectedEngine === 'docker' ? 'podman' : 'docker'}
      >
        {dockerStore.selectedEngine === 'podman' ? 'P' : 'D'}
      </Button>
    </div>
  </div>

  <nav class="flex-1 px-3 py-2 overflow-y-auto">
    <div class="flex flex-col gap-1">
      {#each views as view}
        {@const isActive = page.url.pathname === view.urls}
        {@const Icon = view.icon}
        <a
          href={view.urls}
          class={cn(
            "flex items-center gap-3 px-3 py-2 rounded-lg transition-all duration-200 group relative",
            isActive
              ? "bg-primary/10 text-primary font-semibold shadow-sm"
              : "text-sidebar-foreground/60 hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
          )}
        >
          <Icon class={cn("w-5 h-5 shrink-0 transition-colors", isActive ? "text-primary" : "group-hover:text-sidebar-accent-foreground")} />
          <span class="hidden lg:inline truncate text-sm">{view.label}</span>

          {#if isActive}
            <div class="absolute left-0 w-1 h-5 bg-primary rounded-r-full hidden lg:block"></div>
          {/if}

          {#if view.id === View.Containers && dockerInfo.containers > 0}
            <Badge variant="secondary" class="ml-auto hidden lg:flex h-5 px-1.5 text-[10px] min-w-[20px] justify-center bg-primary/10 text-primary border-none">
              {dockerInfo.containers}
            </Badge>
          {:else if view.id === View.Images && dockerInfo.images > 0}
            <Badge variant="secondary" class="ml-auto hidden lg:flex h-5 px-1.5 text-[10px] min-w-[20px] justify-center bg-primary/10 text-primary border-none">
              {dockerInfo.images}
            </Badge>
          {/if}
        </a>
      {/each}
    </div>
  </nav>

  <div class="p-4 border-t border-sidebar-border mt-auto bg-sidebar/50 backdrop-blur-sm">
    <div class="flex items-center gap-2 mb-1 justify-center lg:justify-start">
      <div class="relative flex items-center justify-center">
        <div class={cn("w-2 h-2 rounded-full", dockerRunning ? "bg-green-500" : "bg-red-500")}></div>
        {#if dockerRunning}
          <div class="absolute w-2 h-2 rounded-full bg-green-500 animate-ping opacity-75"></div>
        {/if}
      </div>
      <span class="text-[10px] lg:text-xs font-bold uppercase tracking-widest text-sidebar-foreground/70 truncate">
        {dockerRunning ? i18n.t('EngineRunning') : i18n.t('EngineStopped')}
      </span>
    </div>
    {#if dockerRunning && dockerInfo.server_version}
      <div class="text-[9px] text-sidebar-foreground/40 font-mono text-center lg:text-left truncate ml-4 lg:ml-4">v{dockerInfo.server_version}</div>
    {/if}
  </div>
</aside>
