<script lang="ts">
  import { invoke, listen } from '$lib/tauri';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import type { PullEvent, CommandResult } from '$lib/types';

  let { show = $bindable(), image = '', onComplete } = $props<{
    show: boolean;
    image?: string;
    onComplete?: () => void;
  }>();

  let imageName = $state(image);
  let isPulling = $state(false);
  let logs = $state<string[]>([]);
  let logContainer = $state<HTMLDivElement>();

  $effect(() => {
    if (logContainer && logs.length > 0) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  });

  $effect(() => {
    if (show && image) {
      imageName = image;
    }
  });

  async function handlePull() {
    if (!imageName) return;

    isPulling = true;
    logs = [`Pulling ${imageName}...` ];

    const unlisten = await listen<PullEvent>('pull-progress', (event) => {
      logs = [...logs, event.payload.status];
    });

    try {
      const result = await dockerStore.invoke<CommandResult>('docker_pull', { image: imageName });
      if (result.success) {
        toastStore.success(`Successfully pulled ${imageName}`);
        setTimeout(() => {
            show = false;
            if (onComplete) onComplete();
            imageName = '';
            logs = [];
        }, 1000);
      } else {
        if (result.error?.includes("terminated")) {
           logs = [...logs, "Pulling cancelled."];
        } else {
           toastStore.error(`Failed to pull ${imageName}: ${result.error}`);
        }
      }
    } catch (e) {
      toastStore.error(`Error pulling image: ${e}`);
    } finally {
      isPulling = false;
      unlisten();
    }
  }

  async function cancelPull() {
    if (!isPulling) return;
    try {
      await invoke('docker_stop_pull');
      toastStore.info('Pulling cancelled');
    } catch (e) {
      console.error('Failed to stop pull', e);
    }
  }

  function close() {
    if (isPulling) return;
    show = false;
    imageName = '';
    logs = [];
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
      <h3 class="text-lg font-bold">Pull Image</h3>
      <div class="py-4">
        <div class="form-control w-full">
          <label class="label" for="imageName">
            <span class="label-text">Image Name (e.g. nginx:latest)</span>
          </label>
          <div class="flex gap-2">
            <input
              id="imageName"
              type="text"
              placeholder="repository:tag"
              class="input input-bordered w-full"
              bind:value={imageName}
              disabled={isPulling}
              onkeydown={(e) => e.key === 'Enter' && handlePull()}
            />
            <button
              class="btn btn-primary"
              onclick={handlePull}
              disabled={isPulling || !imageName}
            >
              {#if isPulling}
                <span class="loading loading-spinner"></span>
              {/if}
              Pull
            </button>
          </div>
        </div>

        {#if logs.length > 0}
          <div class="mt-4">
            <div
              bind:this={logContainer}
              class="bg-base-300 rounded p-4 h-64 overflow-y-auto font-mono text-xs whitespace-pre-wrap"
            >
              {#each logs as log}
                <div class="border-b border-base-100 py-1">{log}</div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
      <div class="modal-action">
        {#if isPulling}
          <button class="btn btn-error btn-outline" onclick={cancelPull}>Cancel Pull</button>
        {/if}
        <button class="btn" onclick={close} disabled={isPulling}>Close</button>
      </div>
    </div>
    <div class="modal-backdrop" role="presentation" onclick={close}></div>
  </div>
{/if}
