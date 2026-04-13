<script lang="ts">
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import type { CommandResult } from '$lib/types';
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Terminal, Play, Loader2 } from "lucide-svelte";

  let { show = $bindable(true), containerId, containerName } = $props<{
    show?: boolean;
    containerId: string;
    containerName: string;
  }>();

  let command = $state('');
  let output = $state('');
  let isExecuting = $state(false);

  async function execute() {
    if (!command) return;
    isExecuting = true;
    output = 'Executing...';
    try {
      const res = await dockerStore.invoke<CommandResult>('docker_exec', {
        containerId,
        command
      });
      if (res.success) {
        output = res.output;
      } else {
        output = `Error: ${res.error}\n${res.output}`;
        toastStore.error('Execution failed');
      }
    } catch (e) {
      output = `Error: ${e}`;
      toastStore.error('System error during execution');
    } finally {
      isExecuting = false;
    }
  }

  function close() {
    show = false;
    command = '';
    output = '';
  }
</script>

<Dialog.Root bind:open={show}>
  <Dialog.Content class="sm:max-w-[800px] flex flex-col max-h-[90vh]">
    <Dialog.Header>
      <div class="flex items-center gap-2">
        <Terminal class="w-5 h-5 text-primary" />
        <Dialog.Title>Exec in {containerName}</Dialog.Title>
      </div>
    </Dialog.Header>

    <div class="space-y-4 py-4">
      <div class="flex gap-2">
        <Input
          type="text"
          placeholder="e.g. ls -la"
          bind:value={command}
          onkeydown={(e) => e.key === 'Enter' && execute()}
          disabled={isExecuting}
        />
        <Button
          onclick={execute}
          disabled={isExecuting || !command}
          class="gap-2"
        >
          {#if isExecuting}
            <Loader2 class="w-4 h-4 animate-spin" />
          {:else}
            <Play class="w-4 h-4" />
          {/if}
          Run
        </Button>
      </div>

      <div class="bg-muted text-foreground border p-4 rounded-lg font-mono text-xs min-h-[200px] max-h-[400px] overflow-auto whitespace-pre-wrap shadow-inner">
        {output || 'Command output will appear here...'}
      </div>
    </div>

    <Dialog.Footer>
      <Button variant="ghost" onclick={close}>Close</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
