import {
  createContext,
  useState,
  useEffect,
  useContext,
  ReactNode,
  Dispatch,
  SetStateAction,
} from "react";
import { DeviceSettings } from "./settings/Device";

interface SettingsContextInterface {
  theme: string;
  setTheme: (theme: string) => void;
  deviceSettings: DeviceSettings | undefined;
  setDeviceSettings: Dispatch<SetStateAction<DeviceSettings | undefined>>;
}

export const SettingsContext = createContext<SettingsContextInterface>({
  theme: localStorage.getItem("theme") || "light",
  setTheme: () => {},
  deviceSettings: undefined,
  setDeviceSettings: () => {},
});

export const useSettingsContext = () => useContext(SettingsContext);

export const SettingsProvider = ({ children }: { children: ReactNode }) => {
  const [theme, setTheme] = useState(localStorage.getItem("theme") || "light");
  const [deviceSettings, setDeviceSettings] = useState<
    DeviceSettings | undefined
  >(undefined);

  useEffect(() => {
    const root = window.document.documentElement;
    root.classList.remove("light", "dark");
    root.classList.add(theme);
    document.body.setAttribute("data-theme", theme);
    localStorage.setItem("theme", theme);
  }, [theme]);

  return (
    <SettingsContext.Provider
      value={{ theme, setTheme, deviceSettings, setDeviceSettings }}
    >
      {children}
    </SettingsContext.Provider>
  );
};
