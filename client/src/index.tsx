/* @refresh reload */
import { render } from "solid-js/web";

import "./index.css";
import App from "./App";
import { Router } from "@solidjs/router";
import { DataProvider } from "./store";

render(
  () => (
    <DataProvider>
      <Router>
        <App />
      </Router>
    </DataProvider>
  ),
  document.getElementById("root") as HTMLElement
);
