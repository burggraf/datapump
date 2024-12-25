<script lang="ts">
    import DataInput from '$lib/components/DataInput.svelte';
    import DataOutput from '$lib/components/DataOutput.svelte';
  import { Button } from '$lib/components/ui/button';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import { Settings } from 'lucide-svelte';
  let open = $state(false);
</script>

<div class="container mx-auto p-4 flex">
  <div class="flex justify-end">
    <Button variant="ghost" size="icon" onclick={() => open = true}>
      <Settings class="h-5 w-5" />
    </Button>
  </div>
    <DataInput />
    <DataOutput />
  <SettingsModal bind:open={open} />
</div>