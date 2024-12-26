<script lang="ts">
    import { Root as Card, Header as CardHeader, Title as CardTitle, Content as CardContent } from '$lib/components/ui/card';
    import { Root as Select, Trigger as SelectTrigger, Value as SelectValue, Content as SelectContent, Item as SelectItem } from '$lib/components/ui/select';
    import { Root as Button } from '$lib/components/ui/button';
    import Label from '$lib/components/ui/label/label.svelte';

    let selectedSqliteType = $state<'db' | 'zip' | null>(null);
    let sqliteContents = $state<string | null>(null);

    const exportSqlite = () => {
        if (!sqliteContents) {
            alert('No data to export');
            return;
        }

        if (!selectedSqliteType) {
            alert('Please select a file type to export');
            return;
        }

        const blob = new Blob([sqliteContents], { type: 'application/octet-stream' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `data.${selectedSqliteType}`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }
</script>

<Card>
    <CardHeader>
        <CardTitle>SQLite</CardTitle>
    </CardHeader>
    <CardContent>
        <div class="mb-4">
            <Label for="sqlite-export-as">Export As</Label>
            <Select>
                <SelectTrigger id="sqlite-export-as" class="w-full">
                    <SelectValue placeholder="Select export type" />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem value="db" onclick={() => selectedSqliteType = 'db'}>Database File</SelectItem>
                    <SelectItem value="zip" onclick={() => selectedSqliteType = 'zip'}>Zip Archive</SelectItem>
                </SelectContent>
            </Select>
        </div>
        <Button class="w-full" onclick={exportSqlite}>Export</Button>
    </CardContent>
</Card>