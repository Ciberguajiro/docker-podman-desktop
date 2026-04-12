<script lang="ts">
  import { i18n } from '$lib/stores/i18n.svelte';
  import { invoke, listen } from '$lib/tauri';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import type { LogEvent, CommandResult } from '$lib/types';
  import { onMount } from 'svelte';

  let { show = $bindable(false), containerId, containerName } = $props<{
    show: boolean;
    containerId: string;
    containerName: string;
  }>();

  let logs = $state<LogEvent[]>([]);
  let isLoading = $state(false);
  let tail = $state(100);
  let logSearch = $state('');
  let logContainer = $state<HTMLDivElement>();
  let unlisten: (() => void) | null = null;

  async function startStreaming() {
    if (!containerId) return;
    isLoading = true;
    logs = [];

    if (unlisten) unlisten();

    // Stop any existing stream on the backend before starting a new one
    await invoke('docker_stop_logs_stream');

    unlisten = await listen<LogEvent>('log-event', (event) => {
      logs = [...logs, event.payload];
    });

    try {
      const res = await dockerStore.invoke<CommandResult>('docker_logs_stream', { containerId, tail });
      if (!res.success) {
        toastStore.error(`Failed to stream logs: ${res.error}`);
      }
    } catch (e) {
      toastStore.error(`Failed to fetch logs: ${e}`);
    } finally {
      isLoading = false;
    }
  }

  $effect(() => {
    if (show && containerId) {
      startStreaming();
    } else if (!show && unlisten) {
      unlisten();
      unlisten = null;
    }
  });

  const filteredLogs = $derived.by(() => {
    if (!logSearch) return logs;
    const query = logSearch.toLowerCase();
    return logs.filter(l => l.line.toLowerCase().includes(query));
  });

  $effect(() => {
    if (logContainer && filteredLogs.length > 0) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  });

  async function close() {
    show = false;
    logs = [];
    if (unlisten) {
      unlisten();
      unlisten = null;
    }
    await invoke('docker_stop_logs_stream');
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
    <div class="modal-box w-11/12 max-w-5xl">
      <div class="flex justify-between items-center mb-4">
        <h3 class="font-bold text-lg">{i18n.t('Logs')}: {containerName}</h3>
        <div class="flex items-center gap-2">
          <div class="relative">
            <input
              type="text"
              placeholder={i18n.t('Search')}
              class="input input-bordered input-xs w-48"
              bind:value={logSearch}
            />
            {#if logSearch}
              <button class="absolute right-1 top-1 text-[10px] opacity-50 hover:opacity-100" onclick={() => logSearch = ''}>✕</button>
            {/if}
          </div>
          <label class="label text-xs" for="tail-input">Tail:</label>
          <input
            id="tail-input"
            type="number"
            class="input input-bordered input-xs w-20"
            bind:value={tail}
            min="1"
          />
          <button class="btn btn-ghost btn-xs" onclick={startStreaming} disabled={isLoading}>
            {#if isLoading}
              <span class="loading loading-spinner loading-xs"></span>
            {/if}
            🔄
          </button>
        </div>
      </div>

      <div class="py-2">
        <div
          bind:this={logContainer}
          class="bg-black text-green-400 p-4 rounded-lg overflow-auto max-h-[60vh] font-mono text-xs whitespace-pre-wrap"
        >
          {#if filteredLogs.length > 0}
            {#each filteredLogs as log}
              <div class={log.is_error ? 'text-error' : ''}>{log.line}</div>
            {/each}
          {:else if isLoading}
            <div class="opacity-50">Loading logs...</div>
          {:else}
            <div class="opacity-50">No logs available.</div>
          {/if}
        </div>
      </div>

      <div class="modal-action">
        <button class="btn btn-primary" onclick={close}>{i18n.t('Close')}</button>
      </div>
    </div>
    <div class="modal-backdrop" role="presentation" onclick={close}></div>
  </div>
{/if}
