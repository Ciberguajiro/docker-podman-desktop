<script lang="ts">
  import { i18n } from '$lib/stores/i18n.svelte';
  import { invoke, listen } from '$lib/tauri';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import type { LogEvent, CommandResult } from '$lib/types';
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { ScrollText, Search, RefreshCw, Loader2 } from "lucide-svelte";
  import { cn } from "$lib/utils";

  let { show = $bindable(true), containerId, containerName } = $props<{
    show?: boolean;
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
</script>

<Dialog.Root bind:open={show}>
  <Dialog.Content class="sm:max-w-[900px] flex flex-col max-h-[90vh]">
    <Dialog.Header>
      <div class="flex items-center justify-between mr-8">
        <div class="flex items-center gap-2">
          <ScrollText class="w-5 h-5 text-primary" />
          <Dialog.Title>{i18n.t('Logs')}: {containerName}</Dialog.Title>
        </div>
        <div class="flex items-center gap-3">
          <div class="relative w-48">
            <Search class="absolute left-2 top-2 w-3.5 h-3.5 text-muted-foreground" />
            <Input
              type="text"
              placeholder={i18n.t('Search')}
              class="h-8 pl-8 text-xs"
              bind:value={logSearch}
            />
          </div>
          <div class="flex items-center gap-2">
            <Label for="tail-input" class="text-xs whitespace-nowrap">Tail:</Label>
            <Input
              id="tail-input"
              type="number"
              class="h-8 w-16 text-xs"
              bind:value={tail}
              min="1"
            />
          </div>
          <Button variant="outline" size="icon" class="h-8 w-8" onclick={startStreaming} disabled={isLoading}>
            {#if isLoading}
              <Loader2 class="w-3.5 h-3.5 animate-spin" />
            {:else}
              <RefreshCw class="w-3.5 h-3.5" />
            {/if}
          </Button>
        </div>
      </div>
    </Dialog.Header>

    <div class="flex-1 overflow-auto py-4">
      <div
        bind:this={logContainer}
        class="bg-muted text-foreground p-4 rounded-lg border overflow-auto max-h-[60vh] font-mono text-[11px] leading-relaxed whitespace-pre-wrap shadow-inner"
      >
        {#if filteredLogs.length > 0}
          {#each filteredLogs as log}
            <div class={cn("py-0.5 border-b border-border/10 last:border-0", log.is_error ? 'text-destructive' : '')}>
              {log.line}
            </div>
          {/each}
        {:else if isLoading}
          <div class="flex flex-col items-center justify-center py-20 opacity-40">
            <Loader2 class="w-8 h-8 animate-spin mb-2" />
            <p>Loading logs...</p>
          </div>
        {:else}
          <div class="flex flex-col items-center justify-center py-20 opacity-40 italic">
            <ScrollText class="w-8 h-8 mb-2" />
            <p>No logs available.</p>
          </div>
        {/if}
      </div>
    </div>

    <Dialog.Footer>
      <Button onclick={close}>{i18n.t('Close')}</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
