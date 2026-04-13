<script lang="ts">
  import { settingsStore } from "$lib/stores/settings.svelte";
  import { dockerStore } from "$lib/stores/docker.svelte";
  import { toastStore } from "$lib/stores/toasts.svelte";
  import type { CommandResult } from "$lib/types";
  import { i18n } from "$lib/stores/i18n.svelte";
  import * as Card from "$lib/components/ui/card";
  import { Label } from "$lib/components/ui/label";
  import * as Select from "$lib/components/ui/select";
  import { Switch } from "$lib/components/ui/switch";
  import { Slider } from "$lib/components/ui/slider";
  import { Button } from "$lib/components/ui/button";
  import { Separator } from "$lib/components/ui/separator";
  import { Badge } from "$lib/components/ui/badge";
  import PageHeader from "$lib/components/ui/PageHeader.svelte";
  import Container from "$lib/components/ui/Container.svelte";
  import {
    Languages,
    Palette,
    RefreshCcw,
    Trash2,
    Database,
    HardDrive,
    Network,
    Box,
    Layers,
    Info,
    Terminal,
    Cpu,
    MemoryStick,
    Settings as SettingsIcon
  } from "lucide-svelte";

  const dockerInfo = $derived(dockerStore.info);

  const themes = [
    { value: "light", label: "Light" },
    { value: "dark", label: "Dark" },
  ];

  const languages = [
    { value: "en", label: "English" },
    { value: "es", label: "Español" },
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

<Container>
  <PageHeader
    title={i18n.t("Settings")}
    description="Configure general appearance and behavior."
    icon={SettingsIcon}
  />

  <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <Palette class="w-5 h-5 text-primary" />
          {i18n.t("AppSettings")}
        </Card.Title>
        <Card.Description>
          Configure general appearance and behavior.
        </Card.Description>
      </Card.Header>
      <Card.Content class="space-y-6">
        <div class="space-y-2">
          <Label for="language-select">{i18n.t("Language")}</Label>
          <Select.Root
            type="single"
            value={settingsStore.language}
            onValueChange={(v) => { if (v) settingsStore.language = v; }}
          >
            <Select.Trigger id="language-select">
              <div class="flex items-center gap-2">
                <Languages class="w-4 h-4 opacity-70" />
                {languages.find(l => l.value === settingsStore.language)?.label || settingsStore.language}
              </div>
            </Select.Trigger>
            <Select.Content>
              {#each languages as lang}
                <Select.Item value={lang.value} label={lang.label} />
              {/each}
            </Select.Content>
          </Select.Root>
        </div>

        <div class="space-y-2">
          <Label for="theme-select">{i18n.t("Theme")}</Label>
          <Select.Root
            type="single"
            value={settingsStore.theme}
            onValueChange={(v) => { if (v) settingsStore.theme = v; }}
          >
            <Select.Trigger id="theme-select">
              <div class="flex items-center gap-2">
                <Palette class="w-4 h-4 opacity-70" />
                {themes.find(t => t.value === settingsStore.theme)?.label || settingsStore.theme}
              </div>
            </Select.Trigger>
            <Select.Content>
              {#each themes as theme}
                <Select.Item value={theme.value} label={theme.label} />
              {/each}
            </Select.Content>
          </Select.Root>
        </div>

        <Separator />

        <div class="flex items-center justify-between">
          <div class="space-y-0.5">
            <Label for="auto-refresh">{i18n.t("AutoRefresh")}</Label>
            <p class="text-sm text-muted-foreground">Automatically update data from the engine.</p>
          </div>
          <Switch
            id="auto-refresh"
            bind:checked={settingsStore.autoRefresh}
          />
        </div>

        {#if settingsStore.autoRefresh}
          <div class="space-y-4 pt-2">
            <div class="flex items-center justify-between">
              <Label>{i18n.t("RefreshInterval")} ({i18n.t("Seconds")})</Label>
              <Badge variant="secondary">{settingsStore.refreshInterval}s</Badge>
            </div>
            <Slider
              type="single"
              value={settingsStore.refreshInterval}
              min={5}
              max={60}
              step={5}
              onValueChange={(v: number) => settingsStore.refreshInterval = v}
            />
          </div>
        {/if}
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header>
        <Card.Title class="flex items-center gap-2 text-destructive">
          <Trash2 class="w-5 h-5" />
          {i18n.t("SystemCleanup")}
        </Card.Title>
        <Card.Description>
          {i18n.t("PruneAdvice") || "Remove unused Docker resources to free up space."}
        </Card.Description>
      </Card.Header>
      <Card.Content class="space-y-4">
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <Button variant="outline" class="justify-start gap-2" onclick={() => prune("containers")}>
            <Box class="w-4 h-4" />
            {i18n.t("PruneContainers")}
          </Button>
          <Button variant="outline" class="justify-start gap-2" onclick={() => prune("images")}>
            <Layers class="w-4 h-4" />
            {i18n.t("PruneImages")}
          </Button>
          <Button variant="outline" class="justify-start gap-2" onclick={() => prune("volumes")}>
            <Database class="w-4 h-4" />
            {i18n.t("PruneVolumes")}
          </Button>
          <Button variant="outline" class="justify-start gap-2" onclick={() => prune("networks")}>
            <Network class="w-4 h-4" />
            {i18n.t("PruneNetworks")}
          </Button>
        </div>
        <Separator />
        <Button variant="destructive" class="w-full gap-2" onclick={() => prune("system")}>
          <Trash2 class="w-4 h-4" />
          {i18n.t("PruneSystem") || "Prune Everything"}
        </Button>
      </Card.Content>
    </Card.Root>

    <Card.Root class="md:col-span-2">
      <Card.Header>
        <Card.Title class="flex items-center gap-2">
          <Info class="w-5 h-5 text-primary" />
          {i18n.t("DockerInfo")}
        </Card.Title>
        <Card.Description>
          Information about the connected engine.
        </Card.Description>
      </Card.Header>
      <Card.Content>
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
          <div class="space-y-4">
            <div class="flex items-start gap-3">
              <Terminal class="w-4 h-4 mt-1 text-muted-foreground" />
              <div>
                <p class="text-sm font-medium leading-none mb-1">{i18n.t("ServerVersion")}</p>
                <p class="text-sm text-muted-foreground font-mono">{dockerInfo.server_version}</p>
              </div>
            </div>
            <div class="flex items-start gap-3">
              <Box class="w-4 h-4 mt-1 text-muted-foreground" />
              <div>
                <p class="text-sm font-medium leading-none mb-1">{i18n.t("OSType")}</p>
                <p class="text-sm text-muted-foreground">{dockerInfo.os_type} ({dockerInfo.operating_system})</p>
              </div>
            </div>
            <div class="flex items-start gap-3">
              <Cpu class="w-4 h-4 mt-1 text-muted-foreground" />
              <div>
                <p class="text-sm font-medium leading-none mb-1">{i18n.t("Architecture")} / CPUs</p>
                <p class="text-sm text-muted-foreground">{dockerInfo.architecture} / {dockerInfo.cpus} CPUs</p>
              </div>
            </div>
          </div>

          <div class="space-y-4">
            <div class="flex items-start gap-3">
              <MemoryStick class="w-4 h-4 mt-1 text-muted-foreground" />
              <div>
                <p class="text-sm font-medium leading-none mb-1">{i18n.t("Memory")}</p>
                <p class="text-sm text-muted-foreground">{dockerInfo.memory}</p>
              </div>
            </div>
            <div class="flex items-start gap-3">
              <HardDrive class="w-4 h-4 mt-1 text-muted-foreground" />
              <div>
                <p class="text-sm font-medium leading-none mb-1">{i18n.t("StorageDriver")}</p>
                <p class="text-sm text-muted-foreground font-mono">{dockerInfo.storage_driver}</p>
              </div>
            </div>
            <div class="flex items-start gap-3">
              <RefreshCcw class="w-4 h-4 mt-1 text-muted-foreground" />
              <div>
                <p class="text-sm font-medium leading-none mb-1">{i18n.t("KernelVersion")}</p>
                <p class="text-sm text-muted-foreground font-mono">{dockerInfo.kernel_version}</p>
              </div>
            </div>
          </div>

          <div class="space-y-4">
            <div class="flex items-start gap-3">
              <Box class="w-4 h-4 mt-1 text-muted-foreground" />
              <div>
                <p class="text-sm font-medium leading-none mb-1">Cgroup Driver</p>
                <p class="text-sm text-muted-foreground">{dockerInfo.cgroup_driver} (v{dockerInfo.cgroup_version})</p>
              </div>
            </div>
            <div class="flex items-start gap-3">
              <Terminal class="w-4 h-4 mt-1 text-muted-foreground" />
              <div>
                <p class="text-sm font-medium leading-none mb-1">Logging Driver</p>
                <p class="text-sm text-muted-foreground">{dockerInfo.logging_driver}</p>
              </div>
            </div>
            <div class="flex items-start gap-3">
              <HardDrive class="w-4 h-4 mt-1 text-muted-foreground" />
              <div>
                <p class="text-sm font-medium leading-none mb-1">Root Directory</p>
                <p class="text-sm text-muted-foreground truncate max-w-[200px]" title={dockerInfo.docker_root_dir}>
                  {dockerInfo.docker_root_dir}
                </p>
              </div>
            </div>
          </div>
        </div>
      </Card.Content>
    </Card.Root>
  </div>
</Container>
