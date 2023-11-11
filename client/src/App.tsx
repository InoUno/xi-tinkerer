import Sidebar, { NavItem } from "./components/Sidebar";
import Statusbar from "./components/Statusbar";
import Home from "./components/Home";
import { Routes, Route } from "@solidjs/router";
import { HiSolidChatBubbleLeftRight, HiSolidCog8Tooth, HiSolidPencil, HiSolidPencilSquare, HiSolidShoppingBag, HiSolidUser } from "solid-icons/hi";
import DatTable from "./components/DatTable";
import * as commands from "./bindings";
import Logs from "./components/Logs";

const navItems: NavItem[] = [
  {
    name: "Home",
    path: "/",
    icon: <HiSolidCog8Tooth />,
  },
  { header: "Strings" },
  {
    name: "String tables",
    path: "/strings",
    icon: <HiSolidPencilSquare />,
  },
  {
    name: "Global dialog",
    path: "/global_dialog",
    icon: <HiSolidChatBubbleLeftRight />,
  },

  { header: "By zone" },
  {
    name: "Entity names",
    path: "/entities",
    icon: <HiSolidUser />,
  },
  {
    name: "Dialog",
    path: "/dialog",
    icon: <HiSolidChatBubbleLeftRight />,
  },
  {
    name: "Dialog (2)",
    path: "/dialog2",
    icon: <HiSolidChatBubbleLeftRight />,
  },

  { header: "Misc. data" },
  {
    name: "Items",
    path: "/items",
    icon: <HiSolidShoppingBag />,
  },

];

function App() {
  return (
    <main class="flex flex-col h-screen">
      <div class="flex flex-grow overflow-hidden">
        <Sidebar navItems={navItems} />

        <div class="flex flex-grow flex-col">
          <div class="content flex-grow overflow-y-auto w-full">
            <Routes>
              <Route path="/" component={Home}></Route>

              <Route
                path="/strings"
                component={() => (
                  <DatTable
                    title="Strings"
                    rowsResourceFetcher={() => commands.getStandaloneStringDats()}
                    columns={[{ name: "Name", key: "type" }]}
                    defaultSortColumn="type"
                    toDatDescriptor={(datDescriptor) => datDescriptor}
                  />
                )}
              ></Route>

              <Route
                path="/items"
                component={() => (
                  <DatTable
                    title="Items"
                    rowsResourceFetcher={() => commands.getItemDats()}
                    columns={[{ name: "Name", key: "type" }]}
                    defaultSortColumn="type"
                    toDatDescriptor={(datDescriptor) => datDescriptor}
                  />
                )}
              ></Route>

              <Route
                path="/global_dialog"
                component={() => (
                  <DatTable
                    title="Global Dialog"
                    rowsResourceFetcher={() => commands.getGlobalDialogDats()}
                    columns={[{ name: "Name", key: "type" }]}
                    defaultSortColumn="type"
                    toDatDescriptor={(datDescriptor) => datDescriptor}
                  />
                )}
              ></Route>

              <Route
                path="/entities"
                component={() => (
                  <DatTable
                    title="Entity Names"
                    rowsResourceFetcher={() => commands.getZonesForType({ type: "EntityNames", index: 0 })}
                    columns={[{ name: "Name", key: "name" }, { name: "ID", key: "id" }]}
                    defaultSortColumn="name"
                    toDatDescriptor={(zone) => ({ type: "EntityNames", index: zone.id })}
                  />
                )}
              ></Route>

              <Route
                path="/dialog"
                component={() => (
                  <DatTable
                    title="Dialog"
                    rowsResourceFetcher={() => commands.getZonesForType({ type: "Dialog", index: 0 })}
                    columns={[{ name: "Name", key: "name" }, { name: "ID", key: "id" }]}
                    defaultSortColumn="name"
                    toDatDescriptor={(zone) => ({ type: "Dialog", index: zone.id })}
                  />
                )}
              ></Route>

              <Route
                path="/dialog2"
                component={() => (
                  <DatTable
                    title="Dialog (2)"
                    rowsResourceFetcher={() => commands.getZonesForType({ type: "Dialog2", index: 0 })}
                    columns={[{ name: "Name", key: "name" }, { name: "ID", key: "id" }]}
                    defaultSortColumn="name"
                    toDatDescriptor={(zone) => ({ type: "Dialog2", index: zone.id })}
                  />
                )}
              ></Route>

              <Route
                path="/logs"
                component={Logs}
              ></Route>
            </Routes>
          </div>
          <Statusbar />
        </div>
      </div>
    </main>
  );
}

export default App;
