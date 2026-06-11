<script lang="ts">
  import { i18n } from '$lib/stores/i18n.svelte';
  import { dockerStore } from '$lib/stores/docker.svelte';
  import { goto } from '$app/navigation';

  import { Button } from '$lib/components/ui/button';
  import * as Card from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import PageHeader from '$lib/components/ui/PageHeader.svelte';
  import Container from '$lib/components/ui/Container.svelte';

  import {
    LayoutDashboard,
    Box,
    Layers,
    Database,
    Network,
    Play,
    Download,
    Cpu,
    Info,
    ArrowRight,
  } from 'lucide-svelte';

  const info = $derived(dockerStore.info);

  function navigateTo(path: string) {
    // eslint-disable-next-line svelte/no-navigation-without-resolve
    goto(path);
  }
</script>

<Container>
  <PageHeader
    title={i18n.t('Dashboard')}
    description={i18n.t('DashboardDescription') || 'Overview of your container engine.'}
    icon={LayoutDashboard}
  />

  <div class="grid gap-6 md:grid-cols-2 lg:grid-cols-4 mb-8">
    <Card.Root
      class="group cursor-pointer hover:border-primary/50 transition-colors"
      onclick={() => navigateTo('/containers')}
    >
      <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
        <Card.Title class="text-sm font-medium">{i18n.t('ContainersOverview')}</Card.Title>
        <Box class="h-4 w-4 text-muted-foreground group-hover:text-primary transition-colors" />
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">
          {info.containers}
        </div>
        <div class="flex items-center gap-2 text-xs text-muted-foreground mt-1">
          <Badge variant="default" class="h-4 text-[10px]">{info.containers_running} {i18n.t('RunningContainers') || 'Running'}</Badge>
          <Badge variant="secondary" class="h-4 text-[10px]">{info.containers_stopped} {i18n.t('StoppedContainers') || 'Stopped'}</Badge>
        </div>
      </Card.Content>
    </Card.Root>

    <Card.Root
      class="group cursor-pointer hover:border-primary/50 transition-colors"
      onclick={() => navigateTo('/images')}
    >
      <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
        <Card.Title class="text-sm font-medium">{i18n.t('ImagesOverview')}</Card.Title>
        <Layers class="h-4 w-4 text-muted-foreground group-hover:text-primary transition-colors" />
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold">{info.images}</div>
        <p class="text-xs text-muted-foreground mt-1">{i18n.t('TotalImages') || 'Images'}</p>
      </Card.Content>
    </Card.Root>

    <Card.Root
      class="group cursor-pointer hover:border-primary/50 transition-colors"
      onclick={() => navigateTo('/volumes')}
    >
      <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
        <Card.Title class="text-sm font-medium">{i18n.t('VolumesOverview')}</Card.Title>
        <Database class="h-4 w-4 text-muted-foreground group-hover:text-primary transition-colors" />
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold text-muted-foreground">
          <LayoutDashboard class="h-6 w-6 inline-block" />
        </div>
        <p class="text-xs text-muted-foreground mt-1">{i18n.t('Volumes')}</p>
      </Card.Content>
    </Card.Root>

    <Card.Root
      class="group cursor-pointer hover:border-primary/50 transition-colors"
      onclick={() => navigateTo('/networks')}
    >
      <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
        <Card.Title class="text-sm font-medium">{i18n.t('NetworksOverview')}</Card.Title>
        <Network class="h-4 w-4 text-muted-foreground group-hover:text-primary transition-colors" />
      </Card.Header>
      <Card.Content>
        <div class="text-2xl font-bold text-muted-foreground">
          <LayoutDashboard class="h-6 w-6 inline-block" />
        </div>
        <p class="text-xs text-muted-foreground mt-1">{i18n.t('Networks')}</p>
      </Card.Content>
    </Card.Root>
  </div>

  <div class="grid gap-6 md:grid-cols-2 mb-6">
    <Card.Root>
      <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
        <div class="flex items-center gap-2">
          <Play class="h-4 w-4 text-primary" />
          <Card.Title class="text-base">{i18n.t('QuickActions')}</Card.Title>
        </div>
      </Card.Header>
      <Card.Content class="space-y-3">
        <Button
          class="w-full justify-start gap-2"
          variant="outline"
          onclick={() => navigateTo('/containers')}
        >
          <Box class="h-4 w-4" />
          {i18n.t('NewContainer')}
          <ArrowRight class="h-3.5 w-3.5 ml-auto" />
        </Button>
        <Button
          class="w-full justify-start gap-2"
          variant="outline"
          onclick={() => navigateTo('/images')}
        >
          <Download class="h-4 w-4" />
          {i18n.t('PullImage')}
          <ArrowRight class="h-3.5 w-3.5 ml-auto" />
        </Button>
        <Button
          class="w-full justify-start gap-2"
          variant="outline"
          onclick={() => navigateTo('/marketplaces')}
        >
          <Download class="h-4 w-4" />
          {i18n.t('Marketplace')}
          <ArrowRight class="h-3.5 w-3.5 ml-auto" />
        </Button>
        <Button
          class="w-full justify-start gap-2"
          variant="outline"
          onclick={() => navigateTo('/metrics')}
        >
          <Cpu class="h-4 w-4" />
          {i18n.t('Metrics')}
          <ArrowRight class="h-3.5 w-3.5 ml-auto" />
        </Button>
      </Card.Content>
    </Card.Root>

    <Card.Root>
      <Card.Header class="flex flex-row items-center justify-between space-y-0 pb-2">
        <div class="flex items-center gap-2">
          <Info class="h-4 w-4 text-primary" />
          <Card.Title class="text-base">{i18n.t('EngineSummary') || 'Engine Summary'}</Card.Title>
        </div>
      </Card.Header>
      <Card.Content class="space-y-4">
        <div class="grid grid-cols-2 gap-4">
          <div class="space-y-1">
            <p class="text-xs text-muted-foreground">{i18n.t('EngineVersion') || 'Version'}</p>
            <p class="text-sm font-mono font-medium">{info.server_version || '--'}</p>
          </div>
          <div class="space-y-1">
            <p class="text-xs text-muted-foreground">{i18n.t('OperatingSystem')}</p>
            <p class="text-sm font-medium">{info.operating_system || '--'}</p>
          </div>
          <div class="space-y-1">
            <p class="text-xs text-muted-foreground">{i18n.t('Architecture')}</p>
            <p class="text-sm font-medium">{info.architecture || '--'}</p>
          </div>
          <div class="space-y-1">
            <p class="text-xs text-muted-foreground">{i18n.t('CPUs')}</p>
            <p class="text-sm font-medium">{info.cpus || '--'}</p>
          </div>
          <div class="space-y-1">
            <p class="text-xs text-muted-foreground">{i18n.t('Memory')}</p>
            <p class="text-sm font-medium">{info.memory || '--'}</p>
          </div>
          <div class="space-y-1">
            <p class="text-xs text-muted-foreground">{i18n.t('StorageDriver')}</p>
            <p class="text-sm font-mono font-medium">{info.storage_driver || '--'}</p>
          </div>
          <div class="space-y-1">
            <p class="text-xs text-muted-foreground">{i18n.t('KernelVersion')}</p>
            <p class="text-sm font-mono font-medium truncate">{info.kernel_version || '--'}</p>
          </div>
          <div class="space-y-1">
            <p class="text-xs text-muted-foreground">{i18n.t('DockerRootDir')}</p>
            <p class="text-sm font-mono font-medium truncate">{info.docker_root_dir || '--'}</p>
          </div>
        </div>
      </Card.Content>
    </Card.Root>
  </div>
</Container>
