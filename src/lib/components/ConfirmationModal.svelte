<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { AlertTriangle } from "lucide-svelte";
  import { cn } from "$lib/utils";

  let {
    show = $bindable(true),
    title,
    message,
    onConfirm,
    onCancel,
    confirmText = "Confirm",
    cancelText = "Cancel",
    isDestructive = false,
    showCancel = true,
    children
  } = $props<{
    show?: boolean;
    title: string;
    message: string;
    onConfirm: () => void;
    onCancel?: () => void;
    confirmText?: string;
    cancelText?: string;
    isDestructive?: boolean;
    showCancel?: boolean;
    children?: any;
  }>();

  function handleConfirm() {
    onConfirm();
    show = false;
  }

  function handleCancel() {
    if (onCancel) onCancel();
    show = false;
  }
</script>

<Dialog.Root bind:open={show}>
  <Dialog.Content class="sm:max-w-[425px]">
    <Dialog.Header>
      <div class="flex items-center gap-3 mb-2">
        {#if isDestructive}
          <div class="p-2 bg-destructive/10 rounded-full">
            <AlertTriangle class="w-5 h-5 text-destructive" />
          </div>
        {/if}
        <Dialog.Title class={cn(isDestructive && "text-destructive")}>{title}</Dialog.Title>
      </div>
      <Dialog.Description class="pt-2">
        {message}
      </Dialog.Description>
    </Dialog.Header>

    {#if children}
      <div class="py-4">
        {@render children()}
      </div>
    {/if}

    <Dialog.Footer class="gap-2 sm:gap-0">
      {#if showCancel}
        <Button variant="ghost" onclick={handleCancel}>
          {cancelText}
        </Button>
      {/if}
      <Button
        variant={isDestructive ? "destructive" : "default"}
        onclick={handleConfirm}
      >
        {confirmText}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
