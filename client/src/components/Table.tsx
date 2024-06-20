import {
  For,
  Show,
  createMemo,
  createResource,
  createSignal,
  onMount,
} from "solid-js";
import fusejs from "fuse.js";

interface TableProps<
  T extends { [key in Column]: any },
  Column extends keyof T
> {
  title: string;
  rowsResourceFetcher: () => Promise<T[]>;
  columns: { name: string; key: Column }[];
  defaultSortColumn: Column;
}

function Table<
  T extends { [key in Column]: any },
  Column extends keyof T & string
>({
  title,
  rowsResourceFetcher,
  columns,
  defaultSortColumn,
}: TableProps<T, Column>) {
  const [rowsResource] = createResource(rowsResourceFetcher, {
    initialValue: [],
  });

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
        </div>

        <Show when={!rowsResource.loading} fallback={<div>Loading...</div>}>
          <table class="table-auto">
            <thead>
              <tr>
                {columns.map((col) => (
                  <th
                    class="hover:cursor-pointer"
                    onclick={() => updateSort(col.key)}
                  >
                    {col.name}
                  </th>
                ))}
              </tr>
            </thead>

            <tbody>
              <For each={rows()}>
                {(row) => {
                  return (
                    <tr class="hover:bg-slate-700">
                      {columns.map((col) => (
                        <td>{row[col.key]}</td>
                      ))}
                    </tr>
                  );
                }}
              </For>
            </tbody>
          </table>
        </Show>
      </div>
    </div>
  );
}

export default Table;
