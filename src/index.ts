

import { createDbWorker } from "sql.js-httpvfs";

const workerUrl = new URL(
  "sql.js-httpvfs/dist/sqlite.worker.js",
  import.meta.url
);
const wasmUrl = new URL("sql.js-httpvfs/dist/sql-wasm.wasm", import.meta.url);

async function query(query: string) {
  // I really don't know typescript...
  if (!(window as any).worker) {
    
    const worker = await createDbWorker(
    [
      {
        from: "inline",
        config: {
          serverMode: "full",
          url: "./flights.db",
          requestChunkSize: 4096,
        },
      },
    ],
    workerUrl.toString(),
    wasmUrl.toString()
  );
  
  (window as any).worker = worker;
  }

  const result = await (window as any).worker.db.query(query);

  return result;
}
declare global {
    interface Window { MyNamespace: any; }
}

(window as any).query = query;

// load();
