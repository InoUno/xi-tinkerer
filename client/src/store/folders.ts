import { message } from "@tauri-apps/plugin-dialog";
import { commands } from "../bindings";
import { createEffect, createResource, createSignal } from "solid-js";
import { promptFolder, unwrap } from "../util";

export function createFoldersStore() {
  const [getDatFolder, setDatFolderLocal] = createSignal<string | null>();

  // FFXI DAT folder
  const setDatFolder = (path: string | null) => {
    commands.selectFfxiFolder(path)
      .then((new_path) => {
        setDatFolderLocal(unwrap(new_path));
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

    return commands.selectProjectFolder(path)
      .then((recentFolders) => {
        setRecentProjectFolders(unwrap(recentFolders));
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
  const [appPersistence] = createResource(async () => unwrap(await commands.loadPersistenceData()));

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
