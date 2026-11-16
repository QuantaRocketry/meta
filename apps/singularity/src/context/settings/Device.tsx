import type { Protocol } from "@/generated/qcp/Protocol";
import type { RadioConfig } from "@/generated/qcp/RadioConfig";

/** The two radio packets a qcp device accepts. Each is sent on its own. */
export interface RadioSettings {
  config: RadioConfig;
  protocol: Protocol;
}

export interface DeploymentSettings {
  apogee: boolean;
  main: boolean;
  apogee_delay: number;
  main_altitude: number;
}

/**
 * Everything a device can be configured with. A device has some subset of
 * these, and the Configuration page renders one card per capability it has
 * (see `components/capabilities`).
 */
export interface Capabilities {
  radio: RadioSettings;
  deployment: DeploymentSettings;
}

export type CapabilityKey = keyof Capabilities;

/**
 * The backend (`get_device_variants`) is the source of truth for which
 * devices exist and what each has, so nothing here lists them.
 */
export interface DeviceSettings {
  type: string;
  data: Partial<Capabilities>;
}
