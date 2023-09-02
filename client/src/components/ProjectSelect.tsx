import { For, Show, createSignal } from "solid-js";
import { useData } from "../store";

function ProjectSelect() {
  const {
    folders: {
      getRecentProjectFolders,
      getProjectFolder,
      setProjectFolder,
      promptProjectFolder,
    },
  } = useData();

  const [isLoading, setLoading] = createSignal(false);

  const updateFolder = async (folder: string | null) => {
    setLoading(true);
    await setProjectFolder(folder);
    setLoading(false);
  };

  return (
    <div class="flex flex-col space-y-2">
      <div>
        <h2>Project Folder</h2>
        <i>
          This should point to a folder where the human-readable data files should be exported to and live in.<br />
          This is also where DAT files will be generated from the human-readable data files when requested.
        </i>

        <div>
          <button onclick={() => promptProjectFolder()}>
            Select a project folder
          </button>
          <div>Current project folder:
            {getProjectFolder() ? (
              <span class="text-green-200 px-2">{getProjectFolder()}</span>
            ) : (
              <span class="text-red-200 px-2">None</span>
            )}
            <Show when={isLoading()}>
              <span class="italic">(Loading...)</span>
            </Show>
          </div>

        </div>
      </div>

      <div>
        Recently opened projects:
        <ul>
          <For each={getRecentProjectFolders()} fallback={"None"}>
            {(recentProject) => (
              <li class="clickable" onclick={() => updateFolder(recentProject)}>
                {recentProject}
              </li>
            )}
          </For>
        </ul>
      </div>
    </div>
  );
}

export default ProjectSelect;
