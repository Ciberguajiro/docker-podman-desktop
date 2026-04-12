<script lang="ts">
  import { onMount } from 'svelte';
  import { i18n } from '$lib/stores/i18n.svelte';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { toastStore } from '$lib/stores/toasts.svelte';
  import PullModal from '$lib/components/PullModal.svelte';

  interface HubImage {
    name: string;
    description: string;
    star_count: number;
    is_official: boolean;
    is_automated: boolean;
  }

  let searchQuery = $state('');
  let results = $state<HubImage[]>([]);
  let isLoading = $state(false);
  let showPullModal = $state(false);
  let imageToPull = $state('');

  async function searchImages() {
    if (!searchQuery) return;
    isLoading = true;
    try {
      const data = await dockerStore.invoke<any>('docker_hub_search', { query: searchQuery });
      results = data.results || [];
    } catch (e) {
      console.error('Failed to search images', e);
      toastStore.error('Failed to search images');
    } finally {
      isLoading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      searchImages();
    }
  }

  function pullImage(name: string) {
    imageToPull = name;
    showPullModal = true;
  }
</script>

<div class="p-6 h-full flex flex-col">
  <div class="flex justify-between items-center mb-6 shrink-0">
    <div class="flex items-center gap-4 w-full max-w-2xl">
      <h1 class="text-3xl font-bold whitespace-nowrap">{i18n.t('MarketplaceTitle')}</h1>
      <div class="join w-full">
        <input
          type="text"
          placeholder={i18n.t('SearchMarketplace')}
          class="input input-bordered join-item w-full"
          bind:value={searchQuery}
          onkeydown={handleKeydown}
        />
        <button class="btn btn-primary join-item" onclick={searchImages} disabled={isLoading}>
          {#if isLoading}
            <span class="loading loading-spinner loading-xs"></span>
          {/if}
          🔍
        </button>
      </div>
    </div>
  </div>

  <div class="flex-1 overflow-y-auto">
    {#if results.length === 0 && !isLoading}
      <div class="flex flex-col items-center justify-center h-64 opacity-50">
        <span class="text-6xl mb-4">🐳</span>
        <p class="text-xl">{i18n.t('SearchMarketplace')}</p>
      </div>
    {:else}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each results as img}
          <div class="card bg-base-200 shadow-xl hover:shadow-2xl transition-shadow border border-base-content/5">
            <div class="card-body p-5">
              <div class="flex justify-between items-start gap-2">
                <h2 class="card-title text-primary truncate" title={img.name}>{img.name}</h2>
                {#if img.is_official}
                  <div class="badge badge-secondary badge-sm shrink-0">✓ {i18n.t('Official')}</div>
                {/if}
              </div>
              <p class="text-sm line-clamp-3 min-h-[4.5rem] opacity-80">{img.description || 'No description available'}</p>
              <div class="flex items-center gap-4 mt-2 text-xs opacity-60">
                <div class="flex items-center gap-1">
                  <span>⭐</span> {img.star_count.toLocaleString()}
                </div>
              </div>
              <div class="card-actions justify-end mt-4">
                <button class="btn btn-primary btn-sm" onclick={() => pullImage(img.name)}>
                  ☁️ {i18n.t('Pull')}
                </button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <PullModal
    bind:show={showPullModal}
    image={imageToPull}
  />
</div>
