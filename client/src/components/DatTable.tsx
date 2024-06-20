import {
  For,
  Match,
  Show,
  Switch,
  createMemo,
  createResource,
  createSignal,
  onMount,
} from "solid-js";
import fusejs from "fuse.js";
import { commands, DatDescriptor } from "../bindings";
import { useData } from "../store";
import { unwrap } from "../util";

interface DatTableProps<T extends { [key in Column]: any }, Column extends keyof T> {
  title: string;
  rowsResourceFetcher: () => Promise<T[]>,
  columns: { name: string, key: Column }[],
  defaultSortColumn: Column,
  toDatDescriptor: (t: T) => DatDescriptor,
}

function DatTable<T extends { [key in Column]: any }, Column extends keyof T & string>
  ({ title, rowsResourceFetcher, columns, defaultSortColumn, toDatDescriptor }: DatTableProps<T, Column>) {
  const {
    processing: { canProcess, isProcessing, processing },
    workingFiles: { hasWorkingFile }
  } = useData();

  const [rowsResource] = createResource(rowsResourceFetcher, { initialValue: [] });

  const [sortBy, setSortBy] = createSignal<Column>(defaultSortColumn);
  const [sortAsc, setSortAsc] = createSignal<boolean>(true);
  const [filterBy, setFilterBy] = createSignal<string>("");

  const updateSort = (column: Column) => {
    if (column == sortBy()) {
      setSortAsc(!sortAsc());
    } else {
      setSortBy(column as any);
      setSortAsc(true);
    }
  };

  const fuseIndex = createMemo(() => {
    return new fusejs(rowsResource(), {
      keys: columns.map((col) => col.key),
      threshold: 0.3,
    });
  });

  const rows = () => {
    let sortedRows;
    if (filterBy()) {
      sortedRows = fuseIndex()
        .search(filterBy())
        .map((e) => e.item);
    } else {
      sortedRows = [...rowsResource()];
    }

    sortedRows.sort((a, b) => {
      const aValue = a[sortBy()];
      const bValue = b[sortBy()];
      const dir = sortAsc() ? 1 : -1;
      if (aValue < bValue) {
        return -1 * dir;
      } else if (aValue > bValue) {
        return 1 * dir;
      }
      return 0;
    });

    return sortedRows;
  };

  // Make YAML
  const makeAllYaml = () => {
    if (!canProcess()) {
      return;
    }

    rows().forEach((row) => {
      commands.makeYaml(toDatDescriptor(row));
    });
  };

  const makingYamlCount = createMemo(() => {
    return rows()
      .filter((row) => {
        const descriptor = toDatDescriptor(row);
        const key = "index" in descriptor ? descriptor.index : 0;
        return processing.Yaml?.[descriptor.type]?.[key] == true;
      })
      .length;
  });

  // Making DAT files from YAML
  const makeAllDats = () => {
    if (!canProcess()) {
      return;
    }

    rows().forEach((row) => {
      commands.makeDat(toDatDescriptor(row));
    });
  };

  const makingDatCount = createMemo(() => {
    return rows()
      .filter((row) => {
        const descriptor = toDatDescriptor(row);
        const key = "index" in descriptor ? descriptor.index : 0;
        return processing.Dat?.[descriptor.type]?.[key] == true;
      })
      .length;
  });

  let inputRef: HTMLInputElement;
  onMount(() => {
    inputRef.focus();
  });

  return (
    <div>
      <h1>{title}</h1>
      <hr />

      <div>
        <div class="flex flex-row space-x-5">
          <input
            class="mt-3"
            placeholder="Filter"
            ref={inputRef!}
            oninput={(e) => setFilterBy(e.target.value ?? "")}
          />

          <button
            disabled={makingYamlCount() > 0 || !canProcess()}
            onclick={() => makeAllYaml()}
          >
            Export all DATs
          </button>

          <button
            disabled={makingDatCount() > 0 || !canProcess()}
            onclick={() => makeAllDats()}
          >
            Make all DATs
          </button>
        </div>

        <Show when={!rowsResource.loading} fallback={<div>Loading...</div>}>
          <table class="table-auto">
            <thead>
              <tr>
                {columns.map((col) => <th
                  class="hover:cursor-pointer"
                  onclick={() => updateSort(col.key)}
                >
                  {col.name}
                </th>)}

                <th class="w-40">Export from DAT</th>
                <th class="w-40">Generate DAT</th>
              </tr>
            </thead>

            <tbody>
              <For each={rows()}>
                {(row) => {
                  const descriptor = toDatDescriptor(row);
                  return (
                    <tr class="hover:bg-slate-700">
                      {columns.map((col) => <td>{row[col.key]}</td>)}

                      <Show
                        when={canProcess()}
                        fallback={
                          <td colSpan={2}>
                            <span class="italic text-sm">
                              Select a project and FFXI folder.
                            </span>
                          </td>
                        }
                      >
                        <td>
                          <Switch>
                            <Match when={isProcessing("Yaml", descriptor)}>
                              <span class="italic">Exporting...</span>
                            </Match>

                            <Match when={true}>
                              <span
                                class="clickable"
                                onclick={async () => unwrap(await commands.makeYaml(descriptor))}
                              >
                                Export from DAT
                              </span>
                            </Match>
                          </Switch>
                        </td>
                        <td>
                          <Switch>
                            <Match when={hasWorkingFile(descriptor) && isProcessing("Dat", descriptor)}>
                              <span class="italic">Making...</span>
                            </Match>

                            <Match when={hasWorkingFile(descriptor)}>
                              <span
                                class="clickable"
                                onclick={async () => unwrap(await commands.makeDat(descriptor))}
                              >
                                Make DAT
                              </span>
                            </Match>
                          </Switch>
                        </td>
                      </Show>
                    </tr>
                  )
                }}
              </For>
            </tbody>
          </table>
        </Show>
      </div>
    </div>
  );
}

export default DatTable;
