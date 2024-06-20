import { DatProcessorMessage } from '../bindings';
import { listen } from "@tauri-apps/api/event";
import { createStore } from "solid-js/store";

export interface Log {
    descriptor: string,
    message: string,
    datPath: string,
    isError?: boolean,
}


export function createLogsStore() {
    const [logs, setLogs] = createStore<Log[]>([]);

    listen<DatProcessorMessage>("processing", (event) => {
        const payload = event.payload;

        if (payload.state != "Working") {
            let descriptor = payload.dat_descriptor.type;
            if ("index" in payload.dat_descriptor) {
                descriptor += ` (${payload.dat_descriptor.index})`;
            }

            let message, datPath;
            let isError;
            if ("Finished" in payload.state) {
                if (payload.output_kind == "Dat") {
                    message = "Finished generation";
                } else {
                    message = "Finished export";
                }
                datPath = payload.state.Finished;
            } else if ("Error" in payload.state) {
                message = payload.state.Error;
                isError = true;
            }

            let log: Log = {
                descriptor,
                message: message || "",
                datPath: datPath || "",
                isError
            };
            setLogs(logs.length, log);
        }
    })

    return logs;
}