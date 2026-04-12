<script lang="ts">
  import { invoke } from '$lib/tauri';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import type { CommandResult } from '$lib/types';

  let { show = $bindable(false), containerId, containerName } = $props<{
    show: boolean;
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

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && show) {
      close();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if show}
  <div class="modal modal-open">
    <div class="modal-box w-11/12 max-w-4xl">
      <h3 class="font-bold text-lg">Exec in {containerName}</h3>
      <div class="py-4 space-y-4">
        <div class="flex gap-2">
          <input
            type="text"
            placeholder="e.g. ls -la"
            class="input input-bordered flex-1"
            bind:value={command}
            onkeydown={(e) => e.key === 'Enter' && execute()}
          />
          <button
            class="btn btn-primary"
            onclick={execute}
            disabled={isExecuting || !command}
          >
            {#if isExecuting}<span class="loading loading-spinner loading-xs"></span>{/if}
            Run
          </button>
        </div>

        <div class="bg-black text-green-400 p-4 rounded font-mono text-sm min-h-[200px] max-h-[400px] overflow-auto whitespace-pre-wrap">
          {output || 'Command output will appear here...'}
        </div>
      </div>
      <div class="modal-action">
        <button class="btn" onclick={close}>Close</button>
      </div>
    </div>
    <div class="modal-backdrop" role="presentation" onclick={close}></div>
  </div>
{/if}
