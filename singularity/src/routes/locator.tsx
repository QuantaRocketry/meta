import { SerialControlWidget } from "@/components/header-widgets/serial-control";
import { Button } from "@/components/ui/button";
import { showError } from "@/utils/error";
import Page from "@/utils/page";
import { invoke } from "@tauri-apps/api/core";
import "cesium/Build/Cesium/Widgets/widgets.css";
import { WifiIcon, WifiOffIcon } from "lucide-react";
import { useEffect, useState } from "react";
import OfflineMap from "./locator/offline-map";
import OnlineMap from "./locator/online-map";

export default function Locator() {
  const [pageLoaded, setPageLoaded] = useState<boolean>(false);
  const [isOnline, setIsOnline] = useState<boolean>(false);
  const [ionAccessToken, setIonAccessToken] = useState<string | undefined>(
    undefined,
  );

  const toggle_online = async () => {
    if (!isOnline) {
      let res = await load_token();
      if (res) {
        setIsOnline(true);
      } else {
        showError(
          <p>
            No Cesium Ion access token. Please set a token in the{" "}
            <b>
              <u>
                <a href="#settings">Settings</a>
              </u>
            </b>{" "}
            page.
          </p>,
        );
      }
    } else {
      setIsOnline(false);
    }
  };

  const load_token = async () => {
    try {
      const token = await invoke("get_cesium_ion_token");
      setIonAccessToken(token as string);
      return true;
    } catch (_) {
      console.error("Failed to load token, probably no token in config.");
      return false;
    }
  };

  useEffect(() => {
    const initialOnline = async () => {
      let res = await load_token();
      setIsOnline(res && window.navigator.onLine);
      setPageLoaded(true);
    };

    initialOnline();
  }, []);

  return (
    <Page
      title="Map"
      widgets={[
        <Button
          variant="outline"
          size="icon"
          aria-label="Settings"
          onClick={toggle_online}
        >
          {isOnline ? <WifiIcon className="text-cyan-600" /> : <WifiOffIcon />}
        </Button>,
        <SerialControlWidget />,
      ]}
      loaded={pageLoaded}
    >
      <div className="h-full flex flex-row">
        {isOnline ? (
          <OnlineMap ionAccessToken={ionAccessToken} />
        ) : (
          <OfflineMap />
        )}
      </div>
    </Page>
  );
}
