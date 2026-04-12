<script lang="ts">
  let { show = $bindable(), title, message, onConfirm } = $props<{
    show: boolean;
    title: string;
    message: string;
    onConfirm: () => void;
  }>();

  function handleConfirm() {
    onConfirm();
    show = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && show) {
      show = false;
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if show}
  <div class="modal modal-open">
    <div class="modal-box border border-error border-opacity-20 shadow-2xl">
      <div class="flex items-center gap-3 mb-4 text-error">
        <span class="text-2xl">⚠️</span>
        <h3 class="font-bold text-xl">{title}</h3>
      </div>
      <p class="py-4 text-base-content opacity-90">{message}</p>
      <div class="modal-action flex gap-2">
        <button class="btn btn-ghost" onclick={() => (show = false)}>Cancel</button>
        <button class="btn btn-error" onclick={handleConfirm}>Confirm</button>
      </div>
    </div>
    <div class="modal-backdrop" role="presentation" onclick={() => (show = false)}></div>
  </div>
{/if}
