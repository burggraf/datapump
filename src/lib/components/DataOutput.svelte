<script lang="ts">
    import ExportTabs from './ExportTabs.svelte';
    import SqlQuery from './SqlQuery.svelte';
    import type { DatabaseType } from './types';

    let selectedDatabaseType = $state<'postgres' | 'mysql' | 'sqlserver' | null>(null);
    let databaseConnectionString = $state<{ user?: string, password?: string, host?: string, port?: number, dbname?: string } | null>(null);

    const handleDatabaseTypeChange = (event: CustomEvent<'postgres' | 'mysql' | 'sqlserver' | null>) => {
        selectedDatabaseType = event.detail;
    }

    const handleConnectionStringChange = (event: CustomEvent<{ user?: string, password?: string, host?: string, port?: number, dbname?: string } | null>) => {
        databaseConnectionString = event.detail;
    }

    $effect(() => {
        const savedSettings = localStorage.getItem('databaseSettings');
        if (savedSettings) {
            try {
                const settings = JSON.parse(savedSettings);
                if (settings?.output?.postgres) {
                    databaseConnectionString = settings.output.postgres;
                }
            } catch (e) {
                console.error("Error parsing saved settings", e);
            }
        }
    });

    $effect(() => {
        if (databaseConnectionString) {
            localStorage.setItem('databaseSettings', JSON.stringify({
                output: {
                    postgres: databaseConnectionString
                }
            }));
        }
    });
</script>

<div class="w-1/2 p-4">
    <h2 class="text-2xl font-bold mb-4">Output</h2>
    <ExportTabs 
        selectedDatabaseType={selectedDatabaseType}
        databaseConnectionString={databaseConnectionString}
        on:selectedDatabaseTypeChange={handleDatabaseTypeChange}
        on:databaseConnectionStringChange={handleConnectionStringChange}
    />
    <SqlQuery selectedDatabaseType={selectedDatabaseType} databaseConnectionString={databaseConnectionString} />
</div>