import { open } from "@tauri-apps/api/dialog";

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
