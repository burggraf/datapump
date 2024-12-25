<script lang="ts">
    import {
        Dialog,
        DialogContent,
        DialogDescription,
        DialogHeader,
        DialogTitle,
    } from '$lib/components/ui/dialog/index';
    import { Input } from '$lib/components/ui/input/index';
    import { Label } from '$lib/components/ui/label/index';
    import { Button } from '$lib/components/ui/button/index';

    let { open = $bindable() } = $props();
    let flatFileSampleSize = $state(localStorage.getItem('FLAT-FILE-SAMPLE-SIZE') || '10000');

    function saveSettings() {
        localStorage.setItem('FLAT-FILE-SAMPLE-SIZE', flatFileSampleSize);
        open = false;
    }
</script>

<Dialog bind:open={open}>
    <DialogContent>
        <DialogHeader>
            <DialogTitle>Settings</DialogTitle>
            <DialogDescription>
                Configure application settings.
            </DialogDescription>
        </DialogHeader>
        <div class="grid gap-4 py-4">
            <div class="grid gap-2">
                <Label for="flat-file-sample-size">FLAT-FILE-SAMPLE-SIZE</Label>
                <Input
                    id="flat-file-sample-size"
                    type="number"
                    placeholder="10000"
                    bind:value={flatFileSampleSize}
                />
                <p class="text-sm text-muted-foreground">How many lines to sample when determining the schema of a flat file.</p>
            </div>
        </div>
        <Button onclick={saveSettings}>Save</Button>
    </DialogContent>
</Dialog>