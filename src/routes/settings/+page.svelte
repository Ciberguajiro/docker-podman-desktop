<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { dockerStore } from "$lib/stores/docker.svelte";
  import { toastStore } from "$lib/stores/toasts.svelte";
  import type { CommandResult } from "$lib/types";

  const dockerInfo = dockerStore.info;

  import { i18n } from "$lib/stores/i18n.svelte";

  const themes = [
    "light",
    "dark",
    "cupcake",
    "bumblebee",
    "emerald",
    "corporate",
    "synthwave",
    "retro",
    "cyberpunk",
    "valentine",
    "halloween",
    "garden",
    "forest",
    "aqua",
    "lofi",
    "pastel",
    "fantasy",
    "wireframe",
    "black",
    "luxury",
    "dracula",
    "cmyk",
    "autumn",
    "business",
    "acid",
    "lemonade",
    "night",
    "coffee",
    "winter",
  ];

  const languages = [
    { code: "en", name: "English" },
    { code: "es", name: "Español" },
  ];

  async function prune(type: string) {
    try {
      const res = await dockerStore.invoke<CommandResult>("docker_prune", {
        type_: type,
      });
      if (res.success) {
        toastStore.success(`${i18n.t("Prune")} ${type} successful`);
        dockerStore.triggerRefresh();
      } else {
        toastStore.error(`${i18n.t("Prune")} failed: ${res.error}`);
      }
    } catch (e) {
      toastStore.error(`Error: ${e}`);
    }
  }
</script>

<div class="p-6">
  <h1 class="text-3xl font-bold mb-6">{i18n.t("Settings")}</h1>

  <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
    <div class="card bg-base-200 shadow-xl border border-base-300">
      <div class="card-body">
        <h2 class="card-title text-primary">{i18n.t("AppSettings")}</h2>

        <div class="form-control w-full mt-4">
          <label class="label" for="language-select"
            ><span class="label-text">{i18n.t("Language")}</span></label
          >
          <select
            id="language-select"
            class="select select-bordered w-full"
            bind:value={settingsStore.language}
          >
            {#each languages as lang}
              <option value={lang.code}>{lang.name}</option>
            {/each}
          </select>
        </div>

        <div class="form-control w-full">
          <label class="label" for="auto-refresh-toggle">
            <span class="label-text">{i18n.t("AutoRefresh")}</span>
            <input
              id="auto-refresh-toggle"
              type="checkbox"
              class="toggle toggle-primary"
              bind:checked={settingsStore.autoRefresh}
            />
          </label>
        </div>

        {#if settingsStore.autoRefresh}
          <div class="form-control w-full">
            <label class="label" for="refresh-interval">
              <span class="label-text"
                >{i18n.t("RefreshInterval")} ({i18n.t("Seconds")})</span
              >
              <span class="label-text-alt"
                >{settingsStore.refreshInterval}s</span
              >
            </label>
            <input
              id="refresh-interval"
              type="range"
              min="5"
              max="60"
              step="5"
              class="range range-primary range-sm"
              bind:value={settingsStore.refreshInterval}
            />
            <div class="w-full flex justify-between text-xs px-2 mt-1">
              <span>5s</span>
              <span>30s</span>
              <span>60s</span>
            </div>
          </div>
        {/if}

        <div class="divider"></div>

        <div class="form-control w-full mt-4">
          <label class="label" for="theme-select"
            ><span class="label-text">{i18n.t("Theme")}</span></label
          >
          <select
            id="theme-select"
            class="select select-bordered w-full"
            bind:value={settingsStore.theme}
          >
            {#each themes as theme}
              <option value={theme}>{theme}</option>
            {/each}
          </select>
        </div>
      </div>
    </div>

    <div class="card bg-base-200 shadow-xl border border-base-300">
      <div class="card-body">
        <h2 class="card-title text-error">{i18n.t("SystemCleanup")}</h2>
        <p class="text-xs opacity-70 mb-4 text-pretty">
          {i18n.t("PruneAdvice") ||
            "Remove unused Docker resources to free up space."}
        </p>

        <div class="grid grid-cols-2 gap-2">
          <button
            class="btn btn-outline btn-error btn-sm"
            onclick={() => prune("containers")}
          >
            🗑️ {i18n.t("PruneContainers")}
          </button>
          <button
            class="btn btn-outline btn-error btn-sm"
            onclick={() => prune("images")}
          >
            🖼️ {i18n.t("PruneImages")}
          </button>
          <button
            class="btn btn-outline btn-error btn-sm"
            onclick={() => prune("volumes")}
          >
            💾 {i18n.t("PruneVolumes")}
          </button>
          <button
            class="btn btn-outline btn-error btn-sm"
            onclick={() => prune("networks")}
          >
            🌐 {i18n.t("PruneNetworks")}
          </button>
        </div>

        <div class="divider"></div>
        <button
          class="btn btn-error btn-sm w-full"
          onclick={() => prune("system")}
        >
          🔥 {i18n.t("PruneSystem") || "Prune Everything"}
        </button>
      </div>
    </div>

    <div class="card bg-base-200 shadow-xl border border-base-300">
      <div class="card-body">
        <h2 class="card-title text-secondary">{i18n.t("DockerInfo")}</h2>

        <div class="overflow-x-auto mt-4">
          <table class="table table-xs">
            <tbody>
              <tr
                ><td class="font-bold">{i18n.t("ServerVersion")}</td><td
                  >{dockerInfo.server_version}</td
                ></tr
              >
              <tr
                ><td class="font-bold">{i18n.t("OSType")}</td><td
                  >{dockerInfo.os_type}</td
                ></tr
              >
              <tr
                ><td class="font-bold">{i18n.t("OperatingSystem")}</td><td
                  >{dockerInfo.operating_system}</td
                ></tr
              >
              <tr
                ><td class="font-bold">{i18n.t("Architecture")}</td><td
                  >{dockerInfo.architecture}</td
                ></tr
              >
              <tr
                ><td class="font-bold">{i18n.t("KernelVersion")}</td><td
                  >{dockerInfo.kernel_version}</td
                ></tr
              >
              <tr
                ><td class="font-bold">{i18n.t("CPUs")}</td><td
                  >{dockerInfo.cpus}</td
                ></tr
              >
              <tr
                ><td class="font-bold">{i18n.t("Memory")}</td><td
                  >{dockerInfo.memory}</td
                ></tr
              >
              <tr
                ><td class="font-bold">{i18n.t("StorageDriver")}</td><td
                  >{dockerInfo.storage_driver}</td
                ></tr
              >
              <tr
                ><td class="font-bold">{i18n.t("CgroupDriver")}</td><td
                  >{dockerInfo.cgroup_driver} (v{dockerInfo.cgroup_version})</td
                ></tr
              >
              <tr
                ><td class="font-bold">{i18n.t("LoggingDriver")}</td><td
                  >{dockerInfo.logging_driver}</td
                ></tr
              >
              <tr
                ><td class="font-bold">{i18n.t("DockerRootDir")}</td><td
                  >{dockerInfo.docker_root_dir}</td
                ></tr
              >
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</div>
