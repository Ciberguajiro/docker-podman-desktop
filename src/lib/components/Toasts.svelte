<script lang="ts">
  import { toastStore } from '$lib/stores/toasts.svelte';
</script>

<div class="toast toast-end toast-bottom z-50">
  {#each toastStore.value as toast (toast.id)}
    <div class="alert {
      toast.toast_type === 'Success' ? 'alert-success' :
      toast.toast_type === 'Error' ? 'alert-error' :
      toast.toast_type === 'Warning' ? 'alert-warning' :
      'alert-info'
    } shadow-lg mb-2">
      <div>
        <span>{toast.message}</span>
      </div>
      <button class="btn btn-ghost btn-xs" onclick={() => toastStore.remove(toast.id)}>✕</button>
    </div>
  {/each}
</div>
