<script lang="ts">
  import { toastStore } from '$lib/stores/toasts.svelte';
  import { fly, fade } from 'svelte/transition';
  import { X, CheckCircle2, AlertCircle, Info, AlertTriangle } from "lucide-svelte";
  import { cn } from "$lib/utils";

  const getIcon = (type: string) => {
    switch (type) {
      case 'Success': return CheckCircle2;
      case 'Error': return AlertCircle;
      case 'Warning': return AlertTriangle;
      default: return Info;
    }
  };

  const getStyles = (type: string) => {
    switch (type) {
      case 'Success': return "border-green-500/50 bg-green-500/10 text-green-600 dark:text-green-400";
      case 'Error': return "border-red-500/50 bg-red-500/10 text-red-600 dark:text-red-400";
      case 'Warning': return "border-amber-500/50 bg-amber-500/10 text-amber-600 dark:text-amber-400";
      default: return "border-blue-500/50 bg-blue-500/10 text-blue-600 dark:text-blue-400";
    }
  };
</script>

<div class="fixed bottom-4 right-4 z-[100] flex flex-col gap-2 w-full max-w-sm pointer-events-none">
  {#each toastStore.value as toast (toast.id)}
    {@const Icon = getIcon(toast.toast_type)}
    <div
      in:fly={{ x: 20, duration: 300 }}
      out:fade={{ duration: 200 }}
      class={cn(
        "flex items-start gap-3 p-4 rounded-lg border shadow-lg backdrop-blur-md pointer-events-auto",
        getStyles(toast.toast_type)
      )}
    >
      <Icon class="w-5 h-5 mt-0.5 shrink-0" />
      <div class="flex-1 text-sm font-medium leading-tight pt-0.5">
        {toast.message}
      </div>
      <button
        class="opacity-50 hover:opacity-100 transition-opacity p-0.5 rounded-md hover:bg-black/5 dark:hover:bg-white/5"
        onclick={() => toastStore.remove(toast.id)}
      >
        <X class="w-4 h-4" />
      </button>
    </div>
  {/each}
</div>
