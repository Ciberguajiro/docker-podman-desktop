<script lang="ts">
  import "@/styles/global.css";
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import Toasts from "$lib/components/Toasts.svelte";
  import { i18n } from "$lib/stores/i18n.svelte";
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { dockerStore } from "$lib/stores/docker.svelte";

  let { children } = $props();

  onMount(() => {
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
  class="flex h-screen w-screen overflow-hidden bg-base-300 text-base-content"
  data-theme={settingsStore.value.theme}
>
  <Sidebar
    dockerRunning={dockerStore.isRunning}
    dockerInfo={dockerStore.info}
  />

  <main class="flex-1 overflow-auto bg-base-100 relative">
    {#if !dockerStore.isCliInstalled}
      <div in:fade={{ duration: 200 }} class="hero min-h-full bg-base-200">
        <div class="hero-content text-center">
          <div class="max-w-md">
            <div class="text-6xl mb-5">🚫</div>
            <h1 class="text-3xl font-bold">Docker CLI not found</h1>
            <p class="py-6">
              Please install Docker CLI to use this application.
            </p>
            <button
              class="btn btn-primary"
              onclick={() => dockerStore.checkStatus()}>Retry</button
            >
          </div>
        </div>
      </div>
    {:else if !dockerStore.selectedEngine}
      <div in:fade={{ duration: 200 }} class="hero min-h-full bg-base-200">
        <div class="hero-content text-center">
          <div class="max-w-md">
            <div class="text-6xl mb-5">🚀</div>
            <h1 class="text-3xl font-bold">{i18n.t("Welcome")}</h1>
            <p class="py-6">{i18n.t("SelectEngineMessage")}</p>
            <div class="flex gap-4 justify-center">
              <button
                class="btn btn-primary"
                onclick={() => (dockerStore.selectedEngine = "docker")}
                >Docker</button
              >
              <button
                class="btn btn-secondary"
                onclick={() => (dockerStore.selectedEngine = "podman")}
                >Podman</button
              >
            </div>
          </div>
        </div>
      </div>
    {:else if !dockerStore.isRunning}
      <div in:fade={{ duration: 200 }} class="hero min-h-full bg-base-200">
        <div class="hero-content text-center">
          <div class="max-w-md">
            {#if dockerStore.dockerError
              ?.toLowerCase()
              .includes("permission denied")}
              <div class="text-6xl mb-5">🔐</div>
              <h1 class="text-3xl font-bold">{i18n.t("PermissionDenied")}</h1>
              <p class="py-6">
                {i18n.t("PermissionDeniedMessage")}
                <br /><br />
                <code class="bg-base-300 p-2 rounded"
                  >sudo usermod -aG docker $USER</code
                >
                <br /><br />
                {i18n.t("PermissionAdvice")}
              </p>
            {:else}
              <div class="text-6xl mb-5">⚠️</div>
              <h1 class="text-3xl font-bold">{i18n.t("DockerNotRunning")}</h1>
              <p class="py-6">{i18n.t("DockerNotRunningMessage")}</p>
              {#if dockerStore.dockerError}
                <div class="alert alert-error mb-4">
                  <span>{dockerStore.dockerError}</span>
                </div>
              {/if}
            {/if}
            <button
              class="btn btn-primary"
              onclick={() => dockerStore.checkStatus()}
              >{i18n.t("Retry")}</button
            >
          </div>
        </div>
      </div>
    {:else}
      <div
        in:fly={{ y: 10, duration: 200, delay: 200 }}
        out:fade={{ duration: 200 }}
        class="absolute inset-0 overflow-auto p-0"
      >
        {@render children()}
      </div>
    {/if}
  </main>
</div>

<Toasts />
