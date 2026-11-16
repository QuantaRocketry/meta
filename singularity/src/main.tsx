import React from "react";
import ReactDOM from "react-dom/client";
import { createHashRouter, RouterProvider } from "react-router-dom";
import Layout from "./layout";
import ErrorPage from "./error-page";
import Settings from "./routes/settings";
import { TauriProvider } from "./context/TauriProvider";
import "./styles.css";
import { SettingsProvider } from "./context/SettingsProvider";
import Configuration from "./routes/configuration";
import Terminal from "./routes/terminal";
import Locator from "./routes/locator";
import ErrorProvider from "./utils/error";
import Monitor from "./routes/monitor";
import { TooltipProvider } from "@/components/ui/tooltip";

const router = createHashRouter([
  {
    path: "/",
    element: <Layout />,
    errorElement: <ErrorPage />,
    children: [
      {
        index: true,
        element: <Monitor />,
      },
      {
        path: "/monitor",
        element: <Monitor />,
      },
      {
        path: "/configuration",
        element: <Configuration />,
      },
      {
        path: "/locator",
        element: <Locator />,
      },
      {
        path: "/settings",
        element: <Settings />,
      },
      {
        path: "/terminal",
        element: <Terminal />,
      },
    ],
  },
]);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <TauriProvider>
      <SettingsProvider>
        <TooltipProvider>
          <RouterProvider router={router} />
          <ErrorProvider />
        </TooltipProvider>
      </SettingsProvider>
    </TauriProvider>
  </React.StrictMode>,
);
