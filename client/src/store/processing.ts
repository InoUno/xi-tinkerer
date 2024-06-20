import { createStore } from "solid-js/store";
import { DatDescriptor, DatProcessorOutputKind, DatProcessorMessage } from "../bindings";
import { listen } from "@tauri-apps/api/event";
import { createEffect, createSignal } from "solid-js";
import { createFoldersStore } from "./folders";


type DatDescriptorNames = DatDescriptor["type"];
type ProcessingState = {
  [kind in DatProcessorOutputKind]: {
    [name in DatDescriptorNames]?: { [key: number]: boolean }
  }
}

const defaultProcessingState: ProcessingState = {
  Dat: {},
  Yaml: {}
}

export function createProcessingStore(
  folders: ReturnType<typeof createFoldersStore>
) {
  const [processing, setProcessing] = createStore<ProcessingState>(defaultProcessingState);
  const [totalProcessingCount, setTotalProcessingCount] = createSignal(0);
  const [projectPath, setProjectPath] = createSignal();

  createEffect(() => {
    if (folders.getProjectFolder() != projectPath()) {
      setProjectPath(folders.getProjectFolder());
      setProcessing(defaultProcessingState);
    }
  });

  // Listen for changes to the YAML data files
  listen<DatProcessorMessage>("processing", (event) => {
    const payload = event.payload;
    if (event.payload.state == "Working") {
      setTotalProcessingCount(totalProcessingCount() + 1);
      setProcessing(
        payload.output_kind,
        payload.dat_descriptor.type,
        (_typeObj) => ({
          ["index" in payload.dat_descriptor ? payload.dat_descriptor.index : 0]:
            true
        })
      );
    } else {
      setTotalProcessingCount(totalProcessingCount() - 1);
      setProcessing(
        payload.output_kind,
        payload.dat_descriptor.type,
        (_typeObj) => ({
          ["index" in payload.dat_descriptor ? payload.dat_descriptor.index : 0]:
            undefined
        })
      );
    }
  });

  const canProcess = () => {
    return !!folders.getProjectFolder() && !!folders.getDatFolder();
  };

  return {
    processing,
    totalProcessingCount,
    canProcess,

    isProcessing: (outputKind: DatProcessorOutputKind, descriptor: DatDescriptor): boolean | undefined => {
      const key = "index" in descriptor ? descriptor.index : 0;
      return processing[outputKind]?.[descriptor.type]?.[key];
    },

  };
}
