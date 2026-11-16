import { SerialControlWidget } from "@/components/header-widgets/serial-control";
import { Button } from "@/components/ui/button";
import { ButtonGroup } from "@/components/ui/button-group";
import Page from "@/utils/page";
import { invoke } from "@tauri-apps/api/core";
import { useContext, useEffect, useState } from "react";
import { AiOutlineDownload, AiOutlineUpload } from "react-icons/ai";
import { CapabilityCard } from "@/components/capabilities";
import type { CapabilityKey, DeviceSettings } from "../context/settings/Device";
import { SettingsContext } from "../context/SettingsProvider";
import { showError } from "../utils/error";
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
  const [variants, setVariants] = useState<DeviceSettings[]>([]);

  useEffect(() => {
    invoke<DeviceSettings[]>("get_device_variants")
      .then(setVariants)
      .catch(showError);
  }, []);

  async function selectDevice(type: string) {
    const variant = variants.find((v) => v.type === type);
    if (!variant) return;
    await invoke("set_device_settings", { settings: variant });
    setDeviceSettings(variant);
  }

  return (
    <Select onValueChange={selectDevice} value={deviceSettings?.type}>
      <SelectTrigger>
        <SelectValue placeholder="Select a device" />
      </SelectTrigger>
      <SelectContent position={"popper"}>
        <SelectGroup>
          {variants.map((v) => (
            <SelectItem key={v.type} value={v.type}>
              {v.type}
            </SelectItem>
          ))}
        </SelectGroup>
      </SelectContent>
    </Select>
  );
}

function Options() {
  const { deviceSettings, setDeviceSettings } = useContext(SettingsContext);

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

  const capabilities = Object.keys(deviceSettings.data) as CapabilityKey[];

  return (
    <div className="space-y-4">
      {capabilities.map((capability) => (
        <CapabilityCard
          key={`${deviceSettings.type}-${capability}`}
          capability={capability}
          value={deviceSettings.data[capability]!}
          onChange={(value) =>
            setDeviceSettings((current) =>
              current && {
                ...current,
                data: { ...current.data, [capability]: value },
              },
            )
          }
        />
      ))}
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
