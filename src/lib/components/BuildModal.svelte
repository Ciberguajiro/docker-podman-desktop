<script lang="ts">
  import { listen } from '$lib/tauri';
  import { invoke } from '$lib/tauri';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import type { CommandResult, BuildEvent } from '$lib/types';

  interface Props {
    show: boolean;
    onComplete?: () => void;
  }

  let { show = $bindable(), onComplete }: Props = $props();

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

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && show) {
      close();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if show}
  <div class="modal modal-open">
    <div class="modal-box w-11/12 max-w-3xl">
      <h3 class="font-bold text-lg">Build Image from Dockerfile</h3>

      <div class="py-4 space-y-4">
        <div class="form-control">
          <label class="label" for="build-tag"><span class="label-text">Image Tag (repository:tag)</span></label>
          <input
            id="build-tag"
            type="text"
            placeholder="e.g. my-app:latest"
            class="input input-bordered"
            bind:value={buildTag}
            disabled={isBuilding}
          />
        </div>

        <div class="form-control">
          <label class="label" for="build-path"><span class="label-text">Build Context Path</span></label>
          <input
            id="build-path"
            type="text"
            placeholder="Path to folder containing Dockerfile"
            class="input input-bordered"
            bind:value={buildPath}
            disabled={isBuilding}
          />
          <span class="label-text-alt mt-1 px-1">Relative or absolute path on host machine</span>
        </div>

        {#if buildLogs.length > 0 || isBuilding}
          <div class="form-control">
            <span class="label-text mb-2">Build Logs</span>
            <div
              id="build-logs"
              bind:this={logsContainer}
              class="bg-black text-green-500 font-mono text-xs p-4 h-64 overflow-y-auto rounded-lg"
              role="log"
              aria-live="polite"
            >
              {#each buildLogs as log}
                <div class="whitespace-pre-wrap">{log}</div>
              {/each}
              {#if isBuilding}
                <div class="animate-pulse">Building...</div>
              {/if}
            </div>
          </div>
        {/if}
      </div>

      <div class="modal-action">
        {#if isBuilding}
          <button class="btn btn-error btn-outline" onclick={cancelBuild}>Stop Build</button>
        {/if}
        <button class="btn" onclick={close} disabled={isBuilding}>Cancel</button>
        <button
          class="btn btn-primary"
          onclick={startBuild}
          disabled={isBuilding || !buildTag || !buildPath}
        >
          {#if isBuilding}
            <span class="loading loading-spinner"></span>
            Building
          {:else}
            Build
          {/if}
        </button>
      </div>
    </div>
    <div class="modal-backdrop" role="presentation" onclick={close}></div>
  </div>
{/if}
