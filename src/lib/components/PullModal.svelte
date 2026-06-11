<script lang="ts">
  import { invoke, listen } from '$lib/tauri';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import type { PullEvent, CommandResult } from '$lib/types';
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Download, Loader2, XCircle } from "lucide-svelte";

  let { show = $bindable(true), image = '', onComplete } = $props<{
    show?: boolean;
    image?: string;
    onComplete?: () => void;
  }>();

  let imageName = $state('');
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
    if (!show) {
      imageName = '';
      logs = [];
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
      const result = await dockerStore.invoke<CommandResult>('docker_pull', { image: `docker.io/library/${imageName}` });
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
      toastStore.error(`Failed to stop pull: ${e}`);
    }
  }

  function close() {
    if (isPulling) return;
    show = false;
    imageName = '';
    logs = [];
  }
</script>

<Dialog.Root bind:open={show}>
  <Dialog.Content class="sm:max-w-[80svw] flex flex-col max-h-[90vh]">
    <Dialog.Header>
      <div class="flex items-center gap-2">
        <Download class="w-5 h-5 text-primary" />
        <Dialog.Title>Pull Image</Dialog.Title>
      </div>
    </Dialog.Header>

    <div class="space-y-4 py-4">
      <div class="space-y-2">
        <Label for="imageName">Image Name (e.g. nginx:latest)</Label>
        <div class="flex gap-2">
          <Input
            id="imageName"
            type="text"
            placeholder="repository:tag"
            bind:value={imageName}
            disabled={isPulling}
            onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && handlePull()}
          />
          <Button
            onclick={handlePull}
            disabled={isPulling || !imageName}
          >
            {#if isPulling}
              <Loader2 class="w-4 h-4 mr-2 animate-spin" />
            {/if}
            Pull
          </Button>
        </div>
      </div>

      {#if logs.length > 0}
        <div
          bind:this={logContainer}
          class="bg-muted rounded-lg border p-4 h-64 overflow-y-auto font-mono text-xs whitespace-pre-wrap shadow-inner"
        >
          {#each logs as log (log)}
            <div class="border-b border-border/50 py-1 last:border-0">{log}</div>
          {/each}
        </div>
      {/if}
    </div>

    <Dialog.Footer class="gap-2 sm:gap-0">
      {#if isPulling}
        <Button variant="destructive" onclick={cancelPull} class="gap-2">
          <XCircle class="w-4 h-4" />
          Cancel Pull
        </Button>
      {/if}
      <Button variant="ghost" onclick={close} disabled={isPulling}>
        Close
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
