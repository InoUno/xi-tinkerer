import { open } from "@tauri-apps/plugin-dialog";
import { Result } from "./bindings";

export async function promptFolder(
  setFolder: (path: string | null) => any,
  defaultPath?: string | null
) {
  const selected = (await open({
    multiple: false,
    directory: true,
    defaultPath: defaultPath ?? undefined,
  })) as string | null;

  if (selected) {
    setFolder(selected);
  }
}

export function unwrap<T, E>(result: Result<T, E>): T {
  if (result.status == "error") {
    throw result.error;
  }
  return result.data;
}