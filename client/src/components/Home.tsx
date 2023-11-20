import ProjectSelect from "./ProjectSelect";
import FFXISelect from "./FFXISelect";
import * as commands from '../bindings';
import { useData } from "../store";

function Home() {
  const { processing: { totalProcessingCount } } = useData();

  return (
    <div class="flex flex-col space-y-5">
      <h1>Home</h1>
      <hr />
      <FFXISelect />
      <hr />
      <ProjectSelect />
      <hr />
      <div class="flex flex-row space-x-5">
        <button disabled={totalProcessingCount() > 0} onclick={() => totalProcessingCount() == 0 ? commands.makeAllYamls() : undefined}>
          Make all YAMLs
        </button>
        <button disabled={totalProcessingCount() > 0} onclick={() => totalProcessingCount() == 0 ? commands.makeAllDats() : undefined}>
          Make all DATs
        </button>
      </div>
    </div>
  );
}

export default Home;
