import { Moon, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useSettingsContext } from "@/context/SettingsProvider";

export function ThemeToggle() {
  const { theme, setTheme } = useSettingsContext();
  const isDark = theme === "dark";

  return (
    <Button
      type="button"
      variant="ghost"
      size="icon"
      aria-label={isDark ? "Switch to light mode" : "Switch to dark mode"}
      title={isDark ? "Switch to light mode" : "Switch to dark mode"}
      onClick={() => setTheme(isDark ? "light" : "dark")}
    >
      {isDark ? <Moon /> : <Sun />}
    </Button>
  );
}
