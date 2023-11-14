import { createSignal } from "solid-js";
import { useData } from "../store";


function Logs() {
  const { logs } = useData();

  const [onlyErrors, setOnlyErrors] = createSignal<boolean>(true);

  const logsToShow = () => {
    if (onlyErrors()) {
      return logs.filter((log) => log.isError).reverse()
    } else {
      return logs.slice(0).reverse();
    }
  };

  return (
    <div class="flex flex-col space-y-5">
      <h1>Logs</h1>
      <hr />
      <div>
        <label for="only-errors" class="cursor-pointer select-none">Show only errors
          <input type="checkbox" id="only-errors" style={{ display: "inline-block" }} checked={onlyErrors()} onchange={[setOnlyErrors, !onlyErrors()]} />
        </label>
      </div>

      <table>
        <thead>
          <tr>
            <th>Name</th>
            <th>Message</th>
            <th>DAT path</th>
          </tr>
        </thead>
        <tbody>
          {logsToShow().map((log) =>
            <tr>
              <td>{log.descriptor}</td>
              <td>{log.message}</td>
              <td><span class="font-mono text-xs">{log.datPath}</span></td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}

export default Logs;
