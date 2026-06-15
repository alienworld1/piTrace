import { HashRouter } from "react-router";
import { AppRoutes } from "./app/routes";

function App() {
  return (
    <HashRouter>
      <AppRoutes />
    </HashRouter>
  );
}

export default App;
