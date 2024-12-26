import { invoke } from '@tauri-apps/api/core';

export async function executePostgresQuery(connectionString: string, sqlQuery: string) {
    try {
        const data = await invoke("execute_query", { connectionString, query: sqlQuery });
        return { data, error: null };
    } catch (e: any) {
        return { data: null, error: e.toString() };
    }
}
