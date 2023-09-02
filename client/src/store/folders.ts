import { message } from "@tauri-apps/api/dialog";
import {
  loadPersistenceData,
  selectFfxiFolder,
  selectProjectFolder,
} from "../bindings";
import { createEffect, createResource, createSignal } from "solid-js";
import { promptFolder } from "../util";

export function createFoldersStore() {
  const [getDatFolder, setDatFolderLocal] = createSignal<string | null>();

  // FFXI DAT folder
  const setDatFolder = (path: string | null) => {
    selectFfxiFolder(path)
      .then((new_path) => {
        setDatFolderLocal(new_path);
      })
      .catch((err) => {
        message(err);
        setDatFolderLocal(null);
        console.error(err);
      });
  };

  // Project folder
  const [getProjectFolder, setProjectFolderLocal] = createSignal<
    string | null
  >();

  const setProjectFolder = async (path: string | null) => {
    setProjectFolderLocal(path);

    return selectProjectFolder(path)
      .then((recentFolders) => {
        setRecentProjectFolders(recentFolders);
      })
      .catch((err) => {
        message(err);
        setProjectFolderLocal(null);
        console.error(err);
      });
  };

  const [getRecentProjectFolders, setRecentProjectFolders] = createSignal<
    string[]
  >([]);

  // Load data
  const [appPersistence] = createResource(loadPersistenceData);

  createEffect(() => {
    setProjectFolderLocal(appPersistence()?.recent_projects[0]);
    setRecentProjectFolders(appPersistence()?.recent_projects ?? []);
    setDatFolderLocal(appPersistence()?.ffxi_path);
  });

  const promptDatFolder = () => {
    promptFolder(setDatFolder, getDatFolder());
  };

  const promptProjectFolder = () => {
    promptFolder(setProjectFolder, getProjectFolder());
  };

  return {
    getDatFolder,
    setDatFolder,
    promptDatFolder,

    getProjectFolder,
    setProjectFolder,
    promptProjectFolder,

    getRecentProjectFolders,
  };
}
