import { useContext } from "react";
import { DeviceSettings } from "../context/settings/Device";
import { SettingsContext } from "../context/SettingsProvider";
import { showError } from "./error";
import { Field, FieldGroup, FieldLabel, FieldSeparator } from "@/components/ui/field";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Select, SelectContent, SelectGroup, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";


export function LoraOptions() {
  const { deviceSettings, setDeviceSettings } = useContext(SettingsContext);

  const SF_OPTIONS = [
    7,
    8,
    9,
    10,
    11,
    12,
  ]

  const BW_OPTIONS = [
    { value: 7800, label: "7.8 kHz" },
    { value: 10400, label: "10.4 kHz" },
    { value: 15600, label: "15.6 kHz" },
    { value: 20800, label: "20.8 kHz" },
    { value: 31250, label: "31.25 kHz" },
    { value: 41700, label: "41.7 kHz" },
    { value: 62500, label: "62.5 kHz" },
    { value: 125000, label: "125 kHz" },
    { value: 250000, label: "250 kHz" },
    { value: 500000, label: "500 kHz" },
  ] as const;

  const CR_OPTIONS = [
    { value: 5, label: "4/5" },
    { value: 6, label: "4/6" },
    { value: 7, label: "4/7" },
    { value: 8, label: "4/8" },
  ] as const;

  if (!deviceSettings || !("lora" in deviceSettings.data)) {
    showError("Failed to load LoRa settings.");
    return <Card>
      <CardContent className="text-center">
        <div>Failed to load LoRa settings.</div>
      </CardContent>
    </Card>; // Or handle other cases
  }

  const loraSettings = deviceSettings.data.lora;

  const updateLoRaSetting = (key: keyof typeof loraSettings, value: any) => {
    if (setDeviceSettings) {
      if (deviceSettings && "lora" in deviceSettings.data) {
        setDeviceSettings({
          ...deviceSettings,
          data: {
            lora: {
              ...deviceSettings.data.lora,
              [key]: value,
            },
          },
        } as DeviceSettings);
      }
    }
  };

  return (
    <Card>
      <CardHeader>LoRa Configuration</CardHeader>
      <CardContent>
        <FieldGroup className="w-full max-w-xs">

          {/* Frequency */}
          <Field>
            <FieldLabel htmlFor="input-frequency">Frequency</FieldLabel>
            <Input
              id="input-frequency"
              type="number"
              value={deviceSettings.data.lora.frequency != 0 ? deviceSettings.data.lora.frequency : undefined}
              onChange={(e) => updateLoRaSetting("frequency", e.target.value)}
              placeholder="Enter Frequency..." />
          </Field>

          {/* Spreading Factor */}
          <Field>
            <FieldLabel>Spreading Factor</FieldLabel>
            <Select
              value={deviceSettings.data.lora.spreading_factor != 0 ? deviceSettings.data.lora.spreading_factor.toString() : undefined}
              onValueChange={(value) => {
                updateLoRaSetting("spreading_factor", value);
              }}>
              <SelectTrigger>
                <SelectValue placeholder={"Select Spreading Factor..."} />
              </SelectTrigger>
              <SelectContent position={"popper"}>
                <SelectGroup>
                  {SF_OPTIONS.map((t, index) => (
                    <SelectItem key={index} value={t.toString()}>
                      {t}
                    </SelectItem>
                  ))}
                </SelectGroup>
              </SelectContent>
            </Select>
          </Field>

          {/* Bandwidth */}
          <Field>
            <FieldLabel>Bandwidth</FieldLabel>
            <Select
              value={deviceSettings.data.lora.bandwidth != 0 ? deviceSettings.data.lora.bandwidth.toString() : undefined}
              onValueChange={(value) => {
                updateLoRaSetting("bandwidth", value);
              }}>
              <SelectTrigger>
                <SelectValue placeholder={"Select Bandwidth..."} />
              </SelectTrigger>
              <SelectContent position={"popper"}>
                <SelectGroup>
                  {BW_OPTIONS.map((t, index) => (
                    <SelectItem key={index} value={t.value.toString()}>
                      {t.label}
                    </SelectItem>
                  ))}
                </SelectGroup>
              </SelectContent>
            </Select>
          </Field>

          {/* Coding Rate */}
          <Field>
            <FieldLabel>Coding Rate</FieldLabel>
            <Select
              value={deviceSettings.data.lora.coding_rate != 0 ? deviceSettings.data.lora.coding_rate.toString() : undefined}
              onValueChange={(value) => {
                updateLoRaSetting("coding_rate", value);
              }}>
              <SelectTrigger>
                <SelectValue placeholder={"Select Coding Rate..."} />
              </SelectTrigger>
              <SelectContent position={"popper"}>
                <SelectGroup>
                  {CR_OPTIONS.map((t, index) => (
                    <SelectItem key={index} value={t.value.toString()}>
                      {t.label}
                    </SelectItem>
                  ))}
                </SelectGroup>
              </SelectContent>
            </Select>
          </Field>
        </FieldGroup>
      </CardContent>
    </Card >
  );
}

export function DeploymentOptions() {
  const { deviceSettings, setDeviceSettings } = useContext(SettingsContext);

  if (!deviceSettings || !("deployment" in deviceSettings.data)) {
    showError("Failed to load Deployment settings.");
    return <div>Failed to load Deployment settings.</div>; // Or handle other cases
  }

  const deploymentSettings = deviceSettings.data.deployment;

  const updateDeploymentSetting = (
    key: keyof typeof deploymentSettings,
    value: any
  ) => {
    if (setDeviceSettings) {
      if (deviceSettings && "deployment" in deviceSettings.data) {
        setDeviceSettings({
          ...deviceSettings,
          data: {
            deployment: {
              ...deviceSettings.data.deployment,
              [key]: value,
            },
          },
        } as DeviceSettings);
      }
    }
  };

  return (
    <Card>
      <CardHeader>Deployment Configuration</CardHeader>
      <CardContent>
        <FieldGroup className="w-full max-w-xs">

          {/* Apogee Toggle */}
          <Field orientation="horizontal">
            <FieldLabel htmlFor="apogee-event-toggle">Deploy Drogue at Apogee</FieldLabel>
            <Switch id="apogee-event-toggle"
              checked={deploymentSettings.apogee ? deploymentSettings.apogee : false}
              onCheckedChange={(e) => {
                updateDeploymentSetting("apogee", e);
              }} />
          </Field>

          {/* Apogee Delay */}
          <Field>
            <FieldLabel htmlFor="delay-input">Apogee Delay (seconds)</FieldLabel>
            <Input
              id="delay-input"
              type="number"
              value={(deploymentSettings.apogee_delay && deploymentSettings.apogee_delay != 0) ? deploymentSettings.apogee_delay : undefined}
              onChange={(e) => updateDeploymentSetting("apogee_delay", e.target.value)}
              placeholder="Enter Apogee Deployment Delay..." />
          </Field>

          <FieldSeparator />

          {/* Main Toggle */}
          <Field orientation="horizontal">
            <FieldLabel htmlFor="main-event-toggle">Deploy Main</FieldLabel>
            <Switch id="main-event-toggle"
              checked={deploymentSettings.main ? deploymentSettings.main : false}
              onCheckedChange={(e) => {
                updateDeploymentSetting("main", e);
              }} />
          </Field>

          {/* Main Altitude */}
          <Field>
            <FieldLabel htmlFor="main-alt-input">Main Deployment Altitude (metres)</FieldLabel>
            <Input
              id="main-alt-input"
              type="number"
              value={(deploymentSettings.main_altitude && deploymentSettings.main_altitude != 0) ? deploymentSettings.main_altitude : undefined}
              onChange={(e) => updateDeploymentSetting("main_altitude", e.target.value)}
              placeholder="Enter Main Deployment Altitude..." />
          </Field>
        </FieldGroup>
      </CardContent >
    </Card >
  );
}
