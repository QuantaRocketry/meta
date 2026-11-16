import { SerialControlWidget } from "@/components/header-widgets/serial-control";
import { Button } from "@/components/ui/button";
import { ButtonGroup } from "@/components/ui/button-group";
import Page from "@/utils/page";
import { invoke } from "@tauri-apps/api/core";
import { useContext } from "react";
import { useEffect } from "react";
import { AiOutlineDownload, AiOutlineUpload } from "react-icons/ai";
import { DEVICE_VARIANTS, DeviceSettings } from "../context/settings/Device";
import { SettingsContext } from "../context/SettingsProvider";
import { showError } from "../utils/error";
import { DeploymentOptions, LoraOptions } from "../utils/options";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

export function DeviceSelectorWidget() {
  const { deviceSettings, setDeviceSettings } = useContext(SettingsContext);

  async function setDeviceVariant(p: DeviceSettings["type"]) {
    await invoke("set_device_variant", { device: p as string });
    getDeviceSettings();
  }

  async function getDeviceSettings() {
    invoke("get_device_settings")
      .then((p) => {
        if (p) {
          setDeviceSettings(p as DeviceSettings);
        }
      })
      .catch((e) => {
        console.error(e);
        // setDeviceSelect();
      });
  }

  useEffect(() => {
    if (!deviceSettings) {
      getDeviceSettings();
    }
  }, []);

  return (
    <Select
      onValueChange={(value) => {
        setDeviceVariant(value as DeviceSettings["type"]);
      }}
      value={deviceSettings?.type}
    >
      <SelectTrigger>
        <SelectValue placeholder="Select a device" />
      </SelectTrigger>
      <SelectContent position={"popper"}>
        <SelectGroup>
          {DEVICE_VARIANTS.map((value, index) => (
            <SelectItem key={index} value={value}>
              {value}
            </SelectItem>
          ))}
        </SelectGroup>
      </SelectContent>
    </Select>
  );
}

function Options() {
  const { deviceSettings } = useContext(SettingsContext);

  if (!deviceSettings) {
    return (
      <div className="flex grow h-full justify-center">
        <div className="content-center">
          Please select a device or download device settings from a connected
          device.
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {deviceSettings &&
        deviceSettings.data &&
        "lora" in deviceSettings.data && <LoraOptions />}
      {deviceSettings &&
        deviceSettings.data &&
        "deployment" in deviceSettings.data && <DeploymentOptions />}
    </div>
  );
}

export default function Configuration() {
  const { deviceSettings, setDeviceSettings } = useContext(SettingsContext);

  async function upload() {
    console.log(deviceSettings);
    await invoke("upload_device_settings", { settings: deviceSettings }).catch(
      (e) => {
        showError(e);
      },
    );
  }

  async function download() {
    await invoke("download_device_settings")
      .then((settings) => {
        setDeviceSettings(settings as DeviceSettings);
      })
      .catch((e) => {
        showError(e);
      });
  }

  return (
    <Page
      title="Configuration"
      widgets={[<DeviceSelectorWidget />, <SerialControlWidget />]}
    >
      <Options />
      <ButtonGroup
        style={{ position: "absolute", right: "1.25rem", bottom: "1.25rem" }}
      >
        <Button
          variant={"outline"}
          onClick={() => {
            download();
          }}
        >
          <AiOutlineDownload />
        </Button>
        <Button
          variant={"outline"}
          onClick={() => {
            upload();
          }}
        >
          <AiOutlineUpload />
        </Button>
      </ButtonGroup>
    </Page>
  );
}
