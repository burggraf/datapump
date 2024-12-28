import { analyzeSchema } from './flat.svelte';
import { executeSqliteQuery } from './sqlite.svelte';

async function getFile(sourcePath: string): Promise<File> {
    const response = await fetch(sourcePath);
    if (!response.ok) {
        throw new Error(`Failed to fetch file: ${response.status} ${response.statusText}`);
    }
    const blob = await response.blob();
    const file = new File([blob], sourcePath.split('/').pop() || 'unknown');
    return file;
}

export function migrate(sourcePath: string, outputConnectionString: string) {
    return new Promise<void>(async (resolve, reject) => {
        try {
            let tableName = sourcePath.split('/')[sourcePath.split('/').length - 1];
            tableName = tableName.replace(/\.[^/.]+$/, '');
            const schema = await analyzeSchema(file);
            console.log('Schema:');
            console.log(schema);
            for (let i = 0; i < schema.length; i++) {
                console.log(schema[i]);
            }

            let sql = `CREATE TABLE IF NOT EXISTS ${tableName} (${schema.map((field) => `${field.name} ${field.type}`).join(', ')})`;
            console.log('Creating table:');
            console.log(sql);
            /*
            const createTableQuery = sql;
            const { error: createTableError } = await executeSqliteQuery(outputConnectionString, createTableQuery);
            if (createTableError) {
                reject(createTableError);
                return;
            }
            // Placeholder for actual data insertion
            const insertQuery = `INSERT INTO ${file.name} VALUES (${schema.map(() => '?').join(', ')})`;
            const { error: insertError } = await executeSqliteQuery(outputConnectionString, insertQuery);
            if (insertError) {
                reject(insertError);
                return;
            }
                */
            resolve();
        } catch (error) {
            reject(error);
        }
    });
}