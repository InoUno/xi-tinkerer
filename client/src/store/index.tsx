import { createContext, useContext, FlowProps } from "solid-js";
import { createFoldersStore } from "./folders";
import { createProcessingStore } from "./processing";
import { createWorkingFilesStore } from "./workingFiles";
import { createLogsStore } from "./logs";

function makeDataContext() {
  const folders = createFoldersStore();
  return {
    folders,
    processing: createProcessingStore(folders),
    workingFiles: createWorkingFilesStore(folders),
    logs: createLogsStore(),
  };
}

export type DataContextType = ReturnType<typeof makeDataContext>;
const DataContext = createContext<DataContextType>();

export function DataProvider(props: FlowProps) {
  return (
    <DataContext.Provider value={makeDataContext()}>
      {props.children}
    </DataContext.Provider>
  );
}

export function useData() {
  return useContext(DataContext)!;
}
