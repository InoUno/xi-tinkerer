import { useData } from "../store";

function Statusbar() {
  const {
    folders: {
      getDatFolder,
      setDatFolder,
      getProjectFolder,
      setProjectFolder,
      promptDatFolder,
      promptProjectFolder,
    },
  } = useData();

  return (
    <footer class="flex bg-slate-900 border-t border-t-slate-400 text-slate-100 justify-start p-1 text-sm">
      <div class="flex">
        <div
          class="flex-shrink"
          style={"display: grid; grid-template-columns: min-content auto"}
        >
          <div class="px-1 text-right">Project:</div>

          <div
            class="cursor-pointer"
            onclick={(e) => {
              if (e.ctrlKey) {
                setProjectFolder(null);
              } else {
                promptProjectFolder();
              }
            }}
          >
            {getProjectFolder() ? (
              <div class="text-green-200">{getProjectFolder()}</div>
            ) : (
              <div class="underline text-red-200">
                None. Click to select one.
              </div>
            )}
          </div>
          <div class="px-1 text-right">FFXI:</div>
          <div
            class="cursor-pointer"
            onclick={(e) => {
              if (e.ctrlKey) {
                setDatFolder(null);
              } else {
                promptDatFolder();
              }
            }}
          >
            {getDatFolder() ? (
              <div class="text-green-200">{getDatFolder()}</div>
            ) : (
              <div class="underline text-red-200">
                None. Click here to select.
              </div>
            )}
          </div>
        </div>
      </div>
    </footer>
  );
}

export default Statusbar;
