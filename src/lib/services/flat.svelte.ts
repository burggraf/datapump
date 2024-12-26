import Papa from 'papaparse';

const readFirstLines = async (file: File, numLines: number): Promise<string> => {
  return new Promise((resolve, reject) => {
    const stream = file.stream();
    const reader = stream.getReader();
    const decoder = new TextDecoder();
    let lines = '';
    let currentLineCount = 0;

    const read = async () => {
      try {
        const { done, value } = await reader.read();
        if (done) {
          resolve(lines);
          return;
        }
        const chunk = decoder.decode(value);
        lines += chunk;
        const chunkLines = chunk.split('\n');
        currentLineCount += chunkLines.length - 1;
        if (currentLineCount >= numLines) {
          lines = lines.split('\n').slice(0, numLines).join('\n');
          resolve(lines);
          return;
        }
        read();
      } catch (error) {
        reject(error);
      }
    };
    read();
  });
};

export const analyzeSchema = async (file: File): Promise<{ name: string; type: string }[]> => {
  const sampleSize = Number(localStorage.getItem('FLAT-FILE-SAMPLE-SIZE') || '100000');
  const fileContent = await readFirstLines(file, sampleSize);

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
        console.log('Flat file analysis:', results.meta);
        resolve(schema);
      },
      error: (error: Error) => {
        reject(error);
      },
    });
  });
};
