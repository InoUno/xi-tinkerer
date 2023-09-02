import { useData } from "../store";

function FFXISelect() {
  const {
    folders: { getDatFolder, promptDatFolder },
  } = useData();

  return (
    <div>
      <h2>FFXI Folder</h2>
      <i>
        This should point your FFXI installation. It will be used to read DATs from,
        that can be exported into human-readable files.
      </i>

      <div>
        <button onclick={() => promptDatFolder()}>Select a FFXI folder</button>
        <div>
          Current FFXI folder:
          {getDatFolder() ? (
            <span class="text-green-200 px-2">{getDatFolder()}</span>
          ) : (
            <span class="text-red-200 px-2">None</span>
          )}
        </div>
      </div>
    </div>
  );
}

export default FFXISelect;
