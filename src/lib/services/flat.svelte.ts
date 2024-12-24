export const readFile = async (file: File): Promise<string> => {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      resolve(reader.result as string);
    };
    reader.onerror = reject;
    reader.readAsText(file);
  });
};

import Papa from 'papaparse';

export const analyzeSchema = async (file: File): Promise<{ name: string; type: string }[]> => {
  const fileContent = await readFile(file);

  return new Promise((resolve, reject) => {
    Papa.parse(fileContent, {
      header: true,
      dynamicTyping: true,
      complete: (results) => {
        if (!results.meta.fields) {
          resolve([]);
          return;
        }

        const fieldTypes: { [key: string]: string } = {};
        if (results.data.length > 0) {
          const firstRow = results.data[0] as { [key: string]: any };
          for (const field of results.meta.fields) {
            const value = firstRow[field];
            if (typeof value === 'number') {
              fieldTypes[field] = 'number';
            } else if (typeof value === 'boolean') {
              fieldTypes[field] = 'boolean';
            } else {
              fieldTypes[field] = 'string';
            }
          }
        }

        const schema = results.meta.fields.map((name: string) => ({
          name,
          type: fieldTypes[name] || 'unknown',
        }));
        resolve(schema);
      },
      error: (error: Error) => {
        reject(error);
      },
    });
  });
};
