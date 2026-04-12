<script lang="ts">
  import { i18n } from '$lib/stores/i18n.svelte';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { dockerService } from '$lib/services/docker.service';
  import PullModal from '$lib/components/PullModal.svelte';

  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import * as Card from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import {
    Search,
    ShoppingCart,
    Download,
    Star,
    CheckCircle2,
    RefreshCw
  } from "lucide-svelte";
  import { cn } from '$lib/utils';

  let searchQuery = $state('');
  let results = $state<any[]>([]);
  let isLoading = $state(false);
  let showPullModal = $state(false);
  let pullImageName = $state('');

  async function search() {
    if (!searchQuery || !dockerStore.selectedEngine) return;
    isLoading = true;
    try {
      const data = await dockerService.searchHub(dockerStore.selectedEngine, searchQuery);
      results = data;
    } catch (e) {
      console.error('Failed to search', e);
    } finally {
      isLoading = false;
    }
  }

  function openPull(image: string) {
    pullImageName = image;
    showPullModal = true;
  }
</script>

<div class="h-full flex flex-col bg-background">
  <!-- Header -->
  <header class="border-b bg-card/50 backdrop-blur-sm sticky top-0 z-10">
    <div class="container py-6">
      <div class="flex items-center gap-3 mb-6">
        <div class="p-2 bg-primary/10 rounded-lg">
          <ShoppingCart class="h-6 w-6 text-primary" />
        </div>
        <div>
          <h1 class="text-2xl font-bold tracking-tight">{i18n.t('Marketplace')}</h1>
          <p class="text-xs text-muted-foreground">
            Search and pull images from Docker Hub
          </p>
        </div>
      </div>

      <form class="flex gap-2 max-w-2xl" onsubmit={(e) => { e.preventDefault(); search(); }}>
        <div class="relative flex-1">
          <Search class="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            type="search"
            placeholder={i18n.t('SearchMarketplace') || 'Search images on Docker Hub...'}
            class="pl-9"
            bind:value={searchQuery}
          />
        </div>
        <Button type="submit" disabled={isLoading || !searchQuery}>
          {isLoading ? <RefreshCw class="h-4 w-4 mr-2 animate-spin" /> : null}
          {i18n.t('Search')}
        </Button>
      </form>
    </div>
  </header>

  <!-- Content -->
  <div class="flex-1 overflow-auto">
    <div class="container py-8">
      {#if isLoading}
        <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {#each Array(6) as _}
            <Card.Root class="h-40 animate-pulse bg-muted/20" />
          {/each}
        </div>
      {:else if results.length > 0}
        <div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {#each results as res}
            <Card.Root class="group hover:border-primary/50 transition-colors">
              <Card.Header class="pb-2">
                <div class="flex items-start justify-between">
                  <div class="space-y-1">
                    <Card.Title class="text-base font-bold flex items-center gap-2">
                      {res.name}
                      {#if res.is_official}
                        <CheckCircle2 class="h-3.5 w-3.5 text-blue-500" title="Official" />
                      {/if}
                    </Card.Title>
                    <p class="text-xs text-muted-foreground line-clamp-1">{res.description}</p>
                  </div>
                </div>
              </Card.Header>
              <Card.Content class="pb-3 text-xs">
                <div class="flex items-center gap-3 text-muted-foreground">
                  <span class="flex items-center gap-1">
                    <Star class="h-3 w-3 text-amber-500 fill-amber-500" />
                    {res.star_count}
                  </span>
                  {#if res.is_automated}
                    <Badge variant="secondary" class="text-[10px] h-4">Automated</Badge>
                  {/if}
                </div>
              </Card.Content>
              <Card.Footer class="pt-0">
                <Button variant="outline" size="sm" class="w-full h-8 gap-2 group-hover:bg-primary group-hover:text-primary-foreground transition-colors" onclick={() => openPull(res.name)}>
                  <Download class="h-3.5 w-3.5" />
                  {i18n.t('Pull')}
                </Button>
              </Card.Footer>
            </Card.Root>
          {/each}
        </div>
      {:else}
        <div class="flex flex-col items-center justify-center py-20 text-center">
          <div class="bg-muted p-4 rounded-full mb-4">
            <Search class="h-10 w-10 text-muted-foreground/50" />
          </div>
          <h3 class="text-lg font-medium">Ready to explore?</h3>
          <p class="text-sm text-muted-foreground max-w-xs mt-1">
            Search for your favorite images and start building today.
          </p>
        </div>
      {/if}
    </div>
  </div>
</div>

{#if showPullModal}
<PullModal initialImageName={pullImageName} onClose={() => showPullModal = false} />
{/if}
