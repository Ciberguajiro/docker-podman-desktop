<script lang="ts">
  import { listen } from '$lib/tauri';
  import { invoke } from '$lib/tauri';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import type { CommandResult, BuildEvent } from '$lib/types';
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Hammer, Loader2, XCircle } from "lucide-svelte";

  let { show = $bindable(true), onComplete } = $props<{
    show?: boolean;
    onComplete?: () => void;
  }>();

  let buildPath = $state('.');
  let buildTag = $state('');
  let isBuilding = $state(false);
  let buildLogs = $state<string[]>([]);
  let logsContainer = $state<HTMLDivElement | null>(null);

  $effect(() => {
    if (show) {
      const unlisten = listen<BuildEvent>('build-progress', (event) => {
        buildLogs = [...buildLogs, event.payload.line];
        if (logsContainer) {
          logsContainer.scrollTop = logsContainer.scrollHeight;
        }
      });

      return () => {
        unlisten.then(fn => fn());
      };
    }
  });

  async function startBuild() {
    if (!buildTag) {
      toastStore.error('Please provide a tag for the image');
      return;
    }

    isBuilding = true;
    buildLogs = [];

    try {
      const res = await dockerStore.invoke<CommandResult>('docker_build_image', {
        path: buildPath,
        tag: buildTag
      });

      if (res.success) {
        toastStore.success(`Image ${buildTag} built successfully`);
        if (onComplete) onComplete();
        show = false;
      } else {
        if (res.error?.includes("terminated")) {
          buildLogs = [...buildLogs, "Build cancelled."];
        } else {
          toastStore.error(`Build failed: ${res.error}`);
        }
      }
    } catch (e) {
      toastStore.error(`Error: ${e}`);
    } finally {
      isBuilding = false;
    }
  }

  async function cancelBuild() {
    if (!isBuilding) return;
    try {
      await invoke('docker_stop_build');
      toastStore.info('Build cancelled');
    } catch (e) {
      console.error('Failed to stop build', e);
    }
  }

  function close() {
    if (isBuilding) return;
    show = false;
    buildPath = '.';
    buildTag = '';
    buildLogs = [];
  }
</script>

<Dialog.Root bind:open={show}>
  <Dialog.Content class="sm:max-w-[700px] flex flex-col max-h-[90vh]">
    <Dialog.Header>
      <div class="flex items-center gap-2">
        <Hammer class="w-5 h-5 text-primary" />
        <Dialog.Title>Build Image from Dockerfile</Dialog.Title>
      </div>
    </Dialog.Header>

    <div class="space-y-4 py-4">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div class="space-y-2">
          <Label for="build-tag">Image Tag (repository:tag)</Label>
          <Input
            id="build-tag"
            type="text"
            placeholder="e.g. my-app:latest"
            bind:value={buildTag}
            disabled={isBuilding}
          />
        </div>

        <div class="space-y-2">
          <Label for="build-path">Build Context Path</Label>
          <Input
            id="build-path"
            type="text"
            placeholder="Path to folder containing Dockerfile"
            bind:value={buildPath}
            disabled={isBuilding}
          />
        </div>
      </div>

      {#if buildLogs.length > 0 || isBuilding}
        <div class="space-y-2">
          <Label>Build Logs</Label>
          <div
            bind:this={logsContainer}
            class="bg-muted text-foreground font-mono text-xs p-4 h-64 overflow-y-auto rounded-lg border shadow-inner"
          >
            {#each buildLogs as log}
              <div class="whitespace-pre-wrap py-0.5 border-b border-border/30 last:border-0">{log}</div>
            {/each}
            {#if isBuilding}
              <div class="flex items-center gap-2 mt-2 text-primary font-bold">
                <Loader2 class="w-3 h-3 animate-spin" />
                Building...
              </div>
            {/if}
          </div>
        </div>
      {/if}
    </div>

    <Dialog.Footer class="gap-2 sm:gap-0">
      {#if isBuilding}
        <Button variant="destructive" onclick={cancelBuild} class="gap-2">
          <XCircle class="w-4 h-4" />
          Stop Build
        </Button>
      {/if}
      <Button variant="ghost" onclick={close} disabled={isBuilding}>
        Cancel
      </Button>
      <Button
        onclick={startBuild}
        disabled={isBuilding || !buildTag || !buildPath}
        class="gap-2"
      >
        {#if !isBuilding}
          <Hammer class="w-4 h-4" />
        {/if}
        Build
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
