<script lang="ts">
  import "@/styles/global.css";
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import Toasts from "$lib/components/Toasts.svelte";
  import { i18n } from "$lib/stores/i18n.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { dockerStore } from "$lib/stores/docker.svelte";
  import { updaterService } from "$lib/services/updater.service";
  import { Button } from "$lib/components/ui/button";
  import { Zap, AlertTriangle, Lock, RefreshCw } from "lucide-svelte";

  let { children } = $props();

  onMount(() => {
    // Check for updates on startup
    // We check only if it is not in a dev environment/mocked
    // Actually updater plugin handles this gracefully
    updaterService.checkForUpdates(true);

    const handleKeydown = (e: KeyboardEvent) => {
      // Avoid shortcuts when typing in inputs
      if (
        e.target instanceof HTMLInputElement ||
        e.target instanceof HTMLTextAreaElement ||
        e.target instanceof HTMLSelectElement
      ) {
        return;
      }

      if ((e.ctrlKey || e.altKey || e.metaKey) && e.key === "r") {
        e.preventDefault();
        dockerStore.triggerRefresh();
        return;
      }
    };

    window.addEventListener("keydown", handleKeydown);
    return () => window.removeEventListener("keydown", handleKeydown);
  });

  $effect(() => {
    if (!settingsStore.autoRefresh) return;

    const interval = setInterval(() => {
      if (dockerStore.isRunning) {
        dockerStore.triggerRefresh();
      }
    }, settingsStore.refreshInterval * 1000);

    return () => clearInterval(interval);
  });
</script>

<div
  class="flex h-screen w-screen overflow-hidden bg-background text-foreground"
  class:dark={settingsStore.value.theme === 'dark'}
>
  <Sidebar
    dockerRunning={dockerStore.isRunning}
    dockerInfo={dockerStore.info}
  />

  <main class="flex-1 overflow-auto bg-background relative">
    {#if !dockerStore.isCliInstalled}
      <div in:fade={{ duration: 200 }} class="flex flex-col items-center justify-center min-h-full p-6 text-center">
        <div class="p-4 bg-destructive/10 rounded-full mb-6">
          <AlertTriangle class="w-12 h-12 text-destructive" />
        </div>
          <h1 class="text-3xl font-bold tracking-tight mb-2">{i18n.t('DockerCliNotFound')}</h1>
          <p class="text-muted-foreground mb-8 max-w-md">
            {i18n.t('DockerCliNotFoundMessage')}
          </p>
        <Button
          size="lg"
          onclick={() => dockerStore.checkStatus()}
        >
          <RefreshCw class="w-4 h-4 mr-2" />
          Retry
        </Button>
      </div>
    {:else if !dockerStore.selectedEngine}
      <div in:fade={{ duration: 200 }} class="flex flex-col items-center justify-center min-h-full p-6 text-center">
        <div class="p-4 bg-primary/10 rounded-full mb-6 animate-bounce">
          <Zap class="w-12 h-12 text-primary" />
        </div>
        <h1 class="text-3xl font-bold tracking-tight mb-2">{i18n.t("Welcome")}</h1>
        <p class="text-muted-foreground mb-8 max-w-md">
          {i18n.t("SelectEngineMessage")}
        </p>
        <div class="flex gap-4">
          <Button
            size="lg"
            class="min-w-[140px]"
            onclick={() => (dockerStore.selectedEngine = "docker")}
          >
            Docker
          </Button>
          <Button
            variant="secondary"
            size="lg"
            class="min-w-[140px]"
            onclick={() => (dockerStore.selectedEngine = "podman")}
          >
            Podman
          </Button>
        </div>
      </div>
    {:else if !dockerStore.isRunning}
      <div in:fade={{ duration: 200 }} class="flex flex-col items-center justify-center min-h-full p-6 text-center">
        {#if dockerStore.dockerError?.toLowerCase().includes("permission denied")}
          <div class="p-4 bg-amber-500/10 rounded-full mb-6">
            <Lock class="w-12 h-12 text-amber-500" />
          </div>
          <h1 class="text-3xl font-bold tracking-tight mb-2">{i18n.t("PermissionDenied")}</h1>
          <div class="text-muted-foreground mb-8 max-w-md space-y-4">
            <p>{i18n.t("PermissionDeniedMessage")}</p>
            <div class="bg-muted p-3 rounded-md font-mono text-sm border shadow-inner">
              sudo usermod -aG docker $USER
            </div>
            <p>{i18n.t("PermissionAdvice")}</p>
          </div>
        {:else}
          <div class="p-4 bg-destructive/10 rounded-full mb-6">
            <AlertTriangle class="w-12 h-12 text-destructive" />
          </div>
          <h1 class="text-3xl font-bold tracking-tight mb-2">{i18n.t("DockerNotRunning")}</h1>
          <p class="text-muted-foreground mb-8 max-w-md">
            {i18n.t("DockerNotRunningMessage")}
          </p>
          {#if dockerStore.dockerError}
            <div class="bg-destructive/10 text-destructive border border-destructive/20 p-4 rounded-md mb-8 max-w-md text-sm font-medium">
              {dockerStore.dockerError}
            </div>
          {/if}
        {/if}
        <Button
          size="lg"
          onclick={() => dockerStore.checkStatus()}
        >
          <RefreshCw class="w-4 h-4 mr-2" />
          {i18n.t("Retry")}
        </Button>
      </div>
    {:else}
      <div
        in:fly={{ y: 10, duration: 200, delay: 200 }}
        out:fade={{ duration: 200 }}
        class="absolute inset-0 overflow-auto"
      >
        {@render children()}
      </div>
    {/if}
  </main>
</div>

<Toasts />
