import { createStore } from "solid-js/store";
import { DatDescriptor } from "../bindings";
import { commands, FileNotification } from "../bindings";
import { listen } from "@tauri-apps/api/event";
import { createEffect, createSignal } from "solid-js";
import { createFoldersStore } from "./folders";
import { unwrap } from "../util";

type DatDescriptorNames = DatDescriptor["type"];
type WorkingFilesState = {
  [name in DatDescriptorNames]?: { [key: number]: boolean }
}

export function createWorkingFilesStore(
  folders: ReturnType<typeof createFoldersStore>
) {
  const [workingFiles, setWorkingFiles] = createStore<WorkingFilesState>({});
  const [projectPath, setProjectPath] = createSignal();

  const setWorkingFileFromDescriptor = (descriptor: DatDescriptor, is_delete: boolean) => {
    if (is_delete) {
      console.log("Got delete for", descriptor);
    }

    setWorkingFiles(
      descriptor.type,
      (_type) => ({
        ["index" in descriptor ? descriptor.index : 0]: !is_delete
      })
    );
  };

  createEffect(() => {
    if (folders.getProjectFolder() != projectPath()) {
      setProjectPath(folders.getProjectFolder());
      setWorkingFiles({});

      commands.getWorkingFiles().then((loadedWorkingFiles) => {
        setWorkingFiles({});
        for (const datDescriptor of unwrap(loadedWorkingFiles)) {
          setWorkingFileFromDescriptor(datDescriptor, false);
        }
      });
    }
  });

  // Listen for changes to the YAML data files
  listen<FileNotification>("file-change", (event) => {
    const payload = event.payload;
    setWorkingFileFromDescriptor(payload.dat_descriptor, payload.is_delete);
  });

  return {
    hasWorkingFile: (descriptor: DatDescriptor): boolean => {
      const kind = workingFiles[descriptor.type];
      const key = "index" in descriptor ? descriptor.index : 0;
      return kind?.[key] ?? false;
    }
  };
}
