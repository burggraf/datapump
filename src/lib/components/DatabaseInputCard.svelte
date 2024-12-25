<script lang="ts">
    import { Root as Card, Header as CardHeader, Title as CardTitle, Content as CardContent } from '$lib/components/ui/card';
    import { Root as Input } from '$lib/components/ui/input';
    import { Root as Select, Trigger as SelectTrigger, Value as SelectValue, Content as SelectContent, Item as SelectItem } from '$lib/components/ui/select';
    import { Root as Button } from '$lib/components/ui/button';
    import Label from '$lib/components/ui/label/label.svelte';
    import type { DatabaseType } from './types';

    let {
        selectedDatabaseType,
        onDatabaseTypeChange,
        databaseConnectionString,
        onConnectionStringChange,
        onImport
    } = $props<{
        selectedDatabaseType: DatabaseType | null;
        onDatabaseTypeChange: (type: DatabaseType | null) => void;
        databaseConnectionString: string | null;
        onConnectionStringChange: (value: string | null) => void;
        onImport: () => void;
    }>();
</script>

<Card>
    <CardHeader>
        <CardTitle>Databases</CardTitle>
    </CardHeader>
    <CardContent>
        <div class="mb-4">
            <Label for="remote-db-type">Database Type</Label>
            <Select>
                <SelectTrigger id="remote-db-type" class="w-full">
                    <SelectValue placeholder="Select database type" />
                </SelectTrigger>
                <SelectContent>
                    <SelectItem value="postgres" onclick={() => onDatabaseTypeChange('postgres')}>PostgreSQL</SelectItem>
                    <SelectItem value="mysql" onclick={() => onDatabaseTypeChange('mysql')}>MySQL</SelectItem>
                    <SelectItem value="sqlserver" onclick={() => onDatabaseTypeChange('sqlserver')}>SQL Server</SelectItem>
                </SelectContent>
            </Select>
        </div>
        <div class="mb-4">
            <Label for="remote-db-connection">Connection String</Label>
            <Input id="remote-db-connection" placeholder="Enter connection string" on:input={(e) => onConnectionStringChange((e.target as HTMLInputElement)?.value)} />
        </div>
        <Button class="w-full" onclick={onImport}>Import</Button>
    </CardContent>
</Card>