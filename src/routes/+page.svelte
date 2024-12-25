<script lang="ts">
    import DataInput from '$lib/components/DataInput.svelte';
    import DataOutput from '$lib/components/DataOutput.svelte';
  import { Button } from '$lib/components/ui/button';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import { Settings } from 'lucide-svelte';
  let open = $state(false);
</script>

<header class="bg-gray-100 p-4 flex items-center justify-between">
  <h1 class="text-2xl font-bold text-center flex-grow">Data Pump</h1>
  <Button variant="ghost" size="icon" onclick={() => open = true}>
    <Settings class="h-6 w-6" />
  </Button>
</header>
<div class="container mx-auto p-4 flex">
  <DataInput />
  <DataOutput />
  <SettingsModal bind:open={open} />
</div>