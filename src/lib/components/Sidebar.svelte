<script lang="ts">
  import { View, type DockerInfo } from '$lib/types';
  import { i18n } from '$lib/stores/i18n.svelte';
  import { dockerStore } from '$lib/stores/docker.svelte';

  let { dockerRunning, dockerInfo } = $props<{
    dockerRunning: boolean;
    dockerInfo: DockerInfo;
  }>();

  const views = $derived([
    { urls: "/containers" , id: View.Containers, label: i18n.t('Containers'), icon: '📦' },
    { urls: "/images" , id: View.Images, label: i18n.t('Images'), icon: '🖼️' },
    { urls: "/volumes" , id: View.Volumes, label: i18n.t('Volumes'), icon: '💾' },
    { urls: "/networks" , id: View.Networks, label: i18n.t('Networks'), icon: '🌐' },
    { urls: "/metrics" , id: View.Metrics, label: i18n.t('Metrics'), icon: '📊' },
    { urls: "/marketplaces" , id: View.Marketplace, label: i18n.t('Marketplace'), icon: '🛒' },
    { urls: "/settings" , id: View.Settings, label: i18n.t('Settings'), icon: '⚙️' },
  ]);
</script>

<aside class="w-20 lg:w-64 bg-base-300 border-r border-base-100 flex flex-col h-full transition-all duration-300">
  <div class="p-4 flex flex-col gap-4">
    <div class="flex items-center gap-3">
      <div class="avatar">
        <div class="w-10 rounded-full bg-primary text-primary-content flex items-center justify-center shrink-0">
          <svg width="24" height="24" viewBox="0 0 50 50" fill="currentColor">
            <path d="M45 20c-1.5-1-3.5-1.5-5.5-1 0-2-1-4-2.5-5.5-1.5-1.5-3.5-2.5-5.5-3-2 0-4 .5-5.5 1.5-1-2-2.5-3.5-4.5-4.5-2-1-4-1.5-6-1-2 .5-4 1.5-5.5 3-1.5 1.5-2.5 3-3 5-.5 2-.5 4 0 6H2v5h3c0 3 1 6 3 8.5 2 2.5 4.5 4.5 7.5 5.5 3 1 6 1.5 9 1 3-.5 6-2 8-4 2-2 3.5-4.5 4-7.5h3v-5h-2c1-1.5 1.5-3 1.5-5 0-1.5-.5-3-1.5-4.5-.5 0-1.5-.5-2.5-.5zM15 30h-3v-5h3v5zm5 0h-3v-5h3v5zm5 0h-3v-5h3v5zm5 0h-3v-5h3v5zm5-5c0 1-.5 2-1 3h-2v-5h2c.5 1 1 1.5 1 2z"/>
          </svg>
        </div>
      </div>
      <span class="text-xl font-bold tracking-tight hidden lg:inline">
        {dockerStore.selectedEngine === 'podman' ? 'Podman' : 'Docker'}
      </span>
    </div>

    <div class="form-control w-full hidden lg:block">
      <select
        class="select select-bordered select-sm w-full font-bold"
        bind:value={dockerStore.selectedEngine}
      >
        <option value={null} disabled selected>{i18n.t('SelectEngine')}</option>
        <option value="docker">Docker</option>
        <option value="podman">Podman</option>
      </select>
    </div>
    <div class="lg:hidden flex justify-center">
      <button
        class="btn btn-circle btn-xs btn-outline"
        onclick={() => dockerStore.selectedEngine = dockerStore.selectedEngine === 'docker' ? 'podman' : 'docker'}
      >
        {dockerStore.selectedEngine === 'podman' ? 'P' : 'D'}
      </button>
    </div>
  </div>

  <nav class="flex-1 px-2 py-4 overflow-y-auto">
    <ul class="menu menu-md w-full gap-1 p-0">
      {#each views as view}
        <li class="flex justify-center lg:justify-start">
          <a
          href={view.urls}
            class="flex items-center gap-3 justify-center lg:justify-start w-full transition-all duration-200 active:scale-95 hover:bg-primary/10"
          >
            <span class="text-xl">{view.icon}</span>
            <span class="hidden lg:inline">{view.label}</span>
            {#if view.id === View.Containers && dockerInfo.containers > 0}
              <span class="badge badge-sm badge-primary ml-auto hidden lg:flex">{dockerInfo.containers}</span>
            {:else if view.id === View.Images && dockerInfo.images > 0}
              <span class="badge badge-sm badge-primary ml-auto hidden lg:flex">{dockerInfo.images}</span>
            {/if}
          </a>
        </li>
      {/each}
    </ul>
  </nav>

  <div class="p-4 bg-base-200 border-t border-base-100 mt-auto">
    <div class="flex items-center gap-2 mb-1 justify-center lg:justify-start">
      <span class="badge badge-xs {dockerRunning ? 'badge-success' : 'badge-error'}"></span>
      <span class="text-[10px] lg:text-xs font-semibold uppercase tracking-wider opacity-70 truncate">
        {dockerRunning ? i18n.t('EngineRunning') : i18n.t('EngineStopped')}
      </span>
    </div>
    {#if dockerRunning && dockerInfo.server_version}
      <div class="text-[10px] opacity-50 font-mono">v{dockerInfo.server_version}</div>
    {/if}
  </div>
</aside>
