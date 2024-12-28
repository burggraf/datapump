import { analyzeSchema } from './flat.svelte';
import { executeSqliteQuery } from './sqlite.svelte';
import Papa from 'papaparse';

export function migrate(file: File, outputConnectionString: string) {
    return new Promise<void>(async (resolve, reject) => {
        try {
            let tableName = file.name.replace(/\.[^/.]+$/, '');
            const schema = await analyzeSchema(file);
            console.log('Schema:');
            console.log(schema);
            for (let i = 0; i < schema.length; i++) {
                console.log(schema[i]);
            }
            // need to get path to SQLite file and store as dbPath
            const dbPath = outputConnectionString.replace('sqlite://', '');

            let sql = `CREATE TABLE IF NOT EXISTS ${tableName} (${schema.map((field) => `${field.name} ${field.type}`).join(', ')})`;
            console.log('Creating table:');
            console.log(sql);
            console.log("Output connection string:");
            console.log(outputConnectionString);
            const createTableQuery = sql;
            const { error: createTableError } = await executeSqliteQuery(dbPath, createTableQuery);
            if (createTableError) {
                reject(createTableError);
                return;
            }
            // Placeholder for actual data insertion
            const insertQuery = `INSERT INTO ${tableName} VALUES (${schema.map(() => '?').join(', ')})`;
            console.log('Inserting data:');
            console.log(insertQuery);
            const result = await parseFile(file, 10000000);
            console.log('Parsed data:');
            console.log(result);
            /*
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


export const parseFile = async (file: File, batchSize: number) => {
    console.log('Parsing file:');
    console.log('batchSize:', batchSize);
    return new Promise<void>((resolve, reject) => {
        Papa.parse(file, {
            header: false,
            dynamicTyping: true,
            chunkSize: batchSize,
            /*
            step: (results) => {
                console.log('Parsed batch:');
                console.log(results.data);
            },
            */
            chunk: (chunk) => {
                console.log('got a chunk')
                console.log(chunk.data.length);
                console.log(chunk.data[0]);
            },
            complete: () => {
                console.log('Parsing complete');
                resolve();
            },
            error: (error) => {
                reject(error);
            }
        });
    });
};
