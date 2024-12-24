<script lang="ts">
  import { Button } from "$lib/components/ui/button";

  let count = $state(0);

  function handleClick() {
    count++;
    alert("Button clicked!");
  }
</script>

<Button onclick={handleClick}>Click me</Button>

<p>Count: {count}</p>