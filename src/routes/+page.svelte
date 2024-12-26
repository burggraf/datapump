<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Settings } from 'lucide-svelte';
  import * as Card from "$lib/components/ui/card";
  let open = $state(false);
</script>

<header class="bg-gray-100 p-4 flex items-center justify-between">
  <h1 class="text-2xl font-bold text-center flex-grow">Data Pump</h1>
  <Button variant="ghost" size="icon" onclick={() => open = true}>
    <Settings class="h-6 w-6" />
  </Button>
</header>


<div class="flex gap-4 p-4">
  <Card.Root>
    <Card.Header>
      <Card.Title>Input</Card.Title>
      <Card.Description>Card Description</Card.Description>
    </Card.Header>
    <Card.Content>
      <p>Card Content</p>
    </Card.Content>
    <Card.Footer>
      <p>Card Footer</p>
    </Card.Footer>
  </Card.Root>

  <Card.Root>
    <Card.Header>
      <Card.Title>Output</Card.Title>
      <Card.Description>Card Description</Card.Description>
    </Card.Header>
    <Card.Content>
      <p>Card Content</p>
    </Card.Content>
    <Card.Footer>
      <p>Card Footer</p>
    </Card.Footer>
  </Card.Root>
</div>
