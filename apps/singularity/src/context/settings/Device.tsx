export const DEVICE_VARIANTS = ["Entangler", "Warp", "Wio-L1"] as const;

export interface LoraSettings {
  frequency: number;
  bandwidth: number;
  spreading_factor: number;
  sync_word: number;
  coding_rate: number;
}

export interface DeploymentSettings {
  apogee: boolean;
  main: boolean;
  apogee_delay: number;
  main_altitude: number;
}

export interface EntanglerSettings {
  lora: LoraSettings;
}

export interface WarpSettings {
  deployment: DeploymentSettings;
}

export interface WioL1Settings {
  lora: LoraSettings;
}

export type DeviceSettings =
  | { type: "Entangler"; data: EntanglerSettings }
  | { type: "Warp"; data: WarpSettings }
  | { type: "Wio-L1"; data: WioL1Settings };
