<script lang="ts">
  import { i18n } from '$lib/stores/i18n.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { Copy, Terminal } from "lucide-svelte";

  let { show = $bindable(true), title, data } = $props<{
    show?: boolean;
    title: string;
    data: string;
  }>();

  function close() {
    show = false;
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

<Dialog.Root bind:open={show}>
  <Dialog.Content class="sm:max-w-[800px] max-h-[90vh] flex flex-col">
    <Dialog.Header>
      <div class="flex items-center justify-between mr-8">
        <div class="flex items-center gap-2">
          <Terminal class="w-5 h-5 text-primary" />
          <Dialog.Title>{title}</Dialog.Title>
        </div>
        <Button variant="outline" size="sm" onclick={copyToClipboard} class="gap-2">
          <Copy class="w-4 h-4" />
          {i18n.t('Copy')}
        </Button>
      </div>
    </Dialog.Header>

    <div class="flex-1 overflow-auto py-4">
      <div class="bg-muted p-4 rounded-lg border font-mono text-[11px] leading-relaxed shadow-inner">
        <pre class="whitespace-pre-wrap break-all">{formattedData}</pre>
      </div>
    </div>

    <Dialog.Footer>
      <Button onclick={close}>{i18n.t('Close')}</Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
