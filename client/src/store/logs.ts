import * as commands from '../bindings';
import { listen } from "@tauri-apps/api/event";
import { createStore } from "solid-js/store";

export interface Log {
    descriptor: string,
    message: string,
    isError?: boolean,
}


export function createLogsStore() {
    const [logs, setLogs] = createStore<Log[]>([]);

    listen<commands.DatProcessorMessage>("processing", (event) => {
        const payload = event.payload;

        if (payload.state != "Working") {
            let descriptor = payload.dat_descriptor.type;
            if ("index" in payload.dat_descriptor) {
                descriptor += ` (${payload.dat_descriptor.index})`;
            }

            let message;
            let isError;
            if (payload.state == "Finished") {
                message = "Success";
            } else if ("Error" in payload.state) {
                message = payload.state.Error;
                isError = true;
            }

            let log: Log = {
                descriptor,
                message: message!,
                isError
            };
            setLogs(logs.length, log);
        }
    })

    return logs;
}