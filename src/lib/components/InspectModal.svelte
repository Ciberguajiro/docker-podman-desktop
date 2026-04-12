<script lang="ts">
  import { i18n } from '$lib/stores/i18n.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';

  let { show = $bindable(false), title, data } = $props<{
    show: boolean;
    title: string;
    data: string;
  }>();

  function close() {
    show = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && show) {
      close();
    }
  }

  let formattedData = $derived.by(() => {
    if (!data) return '';
    try {
      const parsed = JSON.parse(data);
      return JSON.stringify(parsed, null, 2);
    } catch (e) {
      return data;
    }
  });

  async function copyToClipboard() {
    try {
      await navigator.clipboard.writeText(formattedData);
      toastStore.success(i18n.t('CopiedToClipboard') || 'Copied to clipboard');
    } catch (err) {
      toastStore.error(i18n.t('FailedToCopy') || 'Failed to copy to clipboard');
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if show}
  <div class="modal modal-open">
    <div class="modal-box w-11/12 max-w-5xl">
      <div class="flex justify-between items-center mb-4">
        <h3 class="font-bold text-lg">{title}</h3>
        <button class="btn btn-ghost btn-sm" onclick={copyToClipboard}>
          📋 {i18n.t('Copy')}
        </button>
      </div>
      <div class="py-4">
        <div class="bg-base-300 p-4 rounded-lg overflow-auto max-h-[60vh]">
          <pre class="text-xs font-mono">{formattedData}</pre>
        </div>
      </div>
      <div class="modal-action">
        <button class="btn btn-primary" onclick={close}>{i18n.t('Close')}</button>
      </div>
    </div>
    <div class="modal-backdrop" role="presentation" onclick={close}></div>
  </div>
{/if}
