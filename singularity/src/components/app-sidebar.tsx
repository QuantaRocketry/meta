import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarFooter,
} from "@/components/ui/sidebar";
import {
  Settings,
  MapPinned,
  ChartSpline,
  TestTubeDiagonal,
} from "lucide-react";
import { ThemeToggle } from "@/components/theme-toggle";
import { AiOutlineUsb } from "react-icons/ai";
import { Link } from "react-router-dom";
import { useLocation } from "react-router-dom";

const items = [
  {
    title: "Monitor",
    url: "/monitor",
    icon: ChartSpline,
  },
  {
    title: "Map",
    url: "/locator",
    icon: MapPinned,
  },
  {
    title: "Configuration",
    url: "/configuration",
    icon: TestTubeDiagonal,
  },
  {
    title: "Terminal",
    url: "/terminal",
    icon: AiOutlineUsb,
  },
];

export function AppSidebar() {
  const location = useLocation();

  return (
    <Sidebar>
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Singularity</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {items.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton
                    asChild
                    isActive={location.pathname === item.url}
                  >
                    <Link to={item.url}>
                      <item.icon />
                      <span>{item.title}</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
      <SidebarFooter>
        <div className="flex items-center gap-1">
          <SidebarMenu className="flex-1">
            <SidebarMenuItem>
              <SidebarMenuButton asChild>
                <Link to="/settings">
                  <Settings />
                  <span>Settings</span>
                </Link>
              </SidebarMenuButton>
            </SidebarMenuItem>
          </SidebarMenu>
          <ThemeToggle />
        </div>
      </SidebarFooter>
    </Sidebar>
  );
}
