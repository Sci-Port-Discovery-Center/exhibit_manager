import {
  HashRouter as Router,
  Routes as PrimitiveRoutes,
  Route,
  Navigate,
} from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { AppSidebar } from "@/components/app-sidebar";
import { SidebarProvider, SidebarInset } from "@/components/ui/sidebar";
import { Toaster } from "react-hot-toast";

import { routes } from "@/routes";

function Routes() {
  return (
    <PrimitiveRoutes>
      <Route path="/" element={<Navigate to="/jotforms" replace />} />
      {routes.map((route) =>
        route.sidebar ? (
          <Route
            key={route.url}
            path={route.url}
            element={
              <Layout>
                <route.screen />
              </Layout>
            }
          />
        ) : (
          <Route key={route.url} path={route.url} element={<route.screen />} />
        )
      )}
    </PrimitiveRoutes>
  );
}

function Layout({ children }: { children: React.ReactNode }) {
  return (
    <div className="h-screen overflow-hidden">
      <SidebarProvider
        style={
          {
            "--sidebar-width": "19rem",
          } as React.CSSProperties
        }
      >
        <AppSidebar routeConfig={routes} />
        <SidebarInset>
          <div className="mt-4">{children}</div>
        </SidebarInset>
      </SidebarProvider>
    </div>
  );
}

let queryClient = new QueryClient();

function App() {
  return (
    <Router>
      <QueryClientProvider client={queryClient}>
        <Routes />
        <Toaster
          position="bottom-right"
          toastOptions={{
            style: {
              background: "#19191F",
              color: "#fff",
            },
          }}
        />
      </QueryClientProvider>
    </Router>
  );
}

export default App;
