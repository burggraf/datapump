<script lang="ts">
    import { Root as Tabs, List as TabsList, Trigger as TabsTrigger, Content as TabsContent } from '$lib/components/ui/tabs';
    import FlatFileExport from './FlatFileExport.svelte';
    import SpreadsheetExport from './SpreadsheetExport.svelte';
    import SqliteExport from './SqliteExport.svelte';
    import DatabaseExport from './DatabaseExport.svelte';
    import type { FileType, DatabaseType } from './types';
    import { createEventDispatcher } from 'svelte';

    const dispatch = createEventDispatcher();

    let selectedExportType = $state< 'flat' | 'spreadsheet' | 'sqlite' | 'database' | null>(null);

    let { selectedDatabaseType, databaseConnectionString } = $props<{
        selectedDatabaseType: 'postgres' | 'mysql' | 'sqlserver' | null,
        databaseConnectionString: { user?: string, password?: string, host?: string, port?: number, dbname?: string } | null
    }>();

    const handleDatabaseTypeChange = (event: CustomEvent<'postgres' | 'mysql' | 'sqlserver' | null>) => {
        selectedDatabaseType = event.detail;
        dispatch('selectedDatabaseTypeChange', selectedDatabaseType);
    }

    const handleConnectionStringChange = (event: CustomEvent<{ user?: string, password?: string, host?: string, port?: number, dbname?: string } | null>) => {
        databaseConnectionString = event.detail;
        dispatch('databaseConnectionStringChange', databaseConnectionString);
    }
</script>

<Tabs value={selectedExportType || 'database'}>
    <TabsList>
        <TabsTrigger value="flat" onclick={() => selectedExportType = 'flat'}>Flat</TabsTrigger>
        <TabsTrigger value="spreadsheet" onclick={() => selectedExportType = 'spreadsheet'}>Spreadsheet</TabsTrigger>
        <TabsTrigger value="sqlite" onclick={() => selectedExportType = 'sqlite'}>SQLite</TabsTrigger>
        <TabsTrigger value="database" onclick={() => selectedExportType = 'database'}>Database</TabsTrigger>
    </TabsList>
    <TabsContent value="flat">
        <FlatFileExport />
    </TabsContent>
    <TabsContent value="spreadsheet">
        <SpreadsheetExport />
    </TabsContent>
    <TabsContent value="sqlite">
        <SqliteExport />
    </TabsContent>
    <TabsContent value="database">
        <DatabaseExport
            selectedDatabaseType={selectedDatabaseType}
            databaseConnectionString={databaseConnectionString}
            on:selectedDatabaseTypeChange={handleDatabaseTypeChange}
            on:databaseConnectionStringChange={handleConnectionStringChange}
        />
    </TabsContent>
</Tabs>