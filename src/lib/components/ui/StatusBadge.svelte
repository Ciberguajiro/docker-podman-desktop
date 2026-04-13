<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import { cn } from "$lib/utils";

  let { status, class: className } = $props<{
    status: string;
    class?: string;
  }>();

  const getStatusColor = (status: string) => {
    const s = status.toLowerCase();
    if (s.includes('running') || s.includes('up') || s.includes('healthy')) return 'bg-green-500/10 text-green-500 border-green-500/20';
    if (s.includes('exited') || s.includes('stop') || s.includes('dead')) return 'bg-red-500/10 text-red-500 border-red-500/20';
    if (s.includes('pause')) return 'bg-yellow-500/10 text-yellow-500 border-yellow-500/20';
    if (s.includes('restarting') || s.includes('create')) return 'bg-blue-500/10 text-blue-500 border-blue-500/20';
    return 'bg-muted text-muted-foreground';
  };
</script>

<Badge variant="outline" class={cn("font-mono text-[10px] uppercase tracking-wider", getStatusColor(status), className)}>
  {status}
</Badge>
