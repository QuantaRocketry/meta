import type { CodingRate } from "@/generated/qcp/CodingRate";
import type { DataCoding } from "@/generated/qcp/DataCoding";
import type { GfskConfig } from "@/generated/qcp/GfskConfig";
import type { GfskCrc } from "@/generated/qcp/GfskCrc";
import type { GfskPulseShape } from "@/generated/qcp/GfskPulseShape";
import type { LoraBandwidth } from "@/generated/qcp/LoraBandwidth";
import type { LoraConfig } from "@/generated/qcp/LoraConfig";
import type { PreamblePattern } from "@/generated/qcp/PreamblePattern";
import type { RadioConfig } from "@/generated/qcp/RadioConfig";
import type { SpreadingFactor } from "@/generated/qcp/SpreadingFactor";
import * as z from "zod";

/*
 * Labels are `Record<Variant, string>`, so adding a variant in Rust and
 * regenerating the types fails to compile here until it has a label.
 */
export const SPREADING_FACTORS: Record<SpreadingFactor, string> = {
  SF5: "SF5",
  SF6: "SF6",
  SF7: "SF7",
  SF8: "SF8",
  SF9: "SF9",
  SF10: "SF10",
  SF11: "SF11",
  SF12: "SF12",
};

export const BANDWIDTHS: Record<LoraBandwidth, string> = {
  Bw7: "7.8 kHz",
  Bw10: "10.4 kHz",
  Bw15: "15.6 kHz",
  Bw20: "20.8 kHz",
  Bw31: "31.25 kHz",
  Bw41: "41.7 kHz",
  Bw62: "62.5 kHz",
  Bw125: "125 kHz",
  Bw250: "250 kHz",
  Bw500: "500 kHz",
};

export const CODING_RATES: Record<CodingRate, string> = {
  Cr45: "4/5",
  Cr46: "4/6",
  Cr47: "4/7",
  Cr48: "4/8",
};

export const PULSE_SHAPES: Record<GfskPulseShape, string> = {
  NoFilter: "None (2-FSK)",
  Bt03: "BT 0.3",
  Bt05: "BT 0.5",
  Bt07: "BT 0.7",
  Bt1: "BT 1.0",
};

export const PREAMBLE_PATTERNS: Record<PreamblePattern, string> = {
  Alternating10: "0xAA",
  Alternating01: "0x55",
};

export const DATA_CODINGS: Record<DataCoding, string> = {
  None: "None",
  Manchester: "Manchester",
  Whitening: "Whitening",
};

export const MODULATIONS = { Lora: "LoRa", Gfsk: "GFSK" } as const;

export const CRC_KINDS = { None: "None", Crc16: "CRC-16" } as const;

function keysOf<K extends string>(record: Record<K, string>) {
  return Object.keys(record) as [K, ...K[]];
}

const hex = (minBytes: number, maxBytes: number) =>
  z
    .string()
    .regex(/^\s*([0-9a-fA-F]{2}\s*){1,}$/, "Hex bytes, e.g. 2D D4")
    .refine((s) => {
      const n = parseHexBytes(s).length;
      return n >= minBytes && n <= maxBytes;
    }, `${minBytes === maxBytes ? minBytes : `${minBytes} to ${maxBytes}`} byte(s)`);

const uint = (max: number) => z.number().int().min(0).max(max);

export const radioFormSchema = z.object({
  frequencyMhz: z.number().min(1).max(6000),
  modulation: z.enum(keysOf(MODULATIONS)),
  lora: z.object({
    spreadingFactor: z.enum(keysOf(SPREADING_FACTORS)),
    bandwidth: z.enum(keysOf(BANDWIDTHS)),
    codingRate: z.enum(keysOf(CODING_RATES)),
    preambleLength: uint(65535),
    implicitHeader: z.boolean(),
    payloadLength: uint(255),
    crcEnabled: z.boolean(),
    invertIq: z.boolean(),
    syncWord: hex(1, 1),
  }),
  gfsk: z.object({
    bitRate: z.number().int().min(1),
    freqDeviation: z.number().int().min(1),
    pulseShape: z.enum(keysOf(PULSE_SHAPES)),
    preamblePattern: z.enum(keysOf(PREAMBLE_PATTERNS)),
    preambleLength: uint(65535),
    syncWord: hex(1, 8),
    variableLength: z.boolean(),
    fixedLength: uint(255),
    crc: z.enum(keysOf(CRC_KINDS)),
    polynomial: hex(2, 2),
    initial: hex(2, 2),
    invertOutput: z.boolean(),
    includeLength: z.boolean(),
    coding: z.enum(keysOf(DATA_CODINGS)),
  }),
});

export type RadioFormValues = z.infer<typeof radioFormSchema>;

export function parseHexBytes(text: string): number[] {
  const digits = text.replace(/\s+/g, "");
  const bytes: number[] = [];
  for (let i = 0; i + 1 < digits.length; i += 2) {
    bytes.push(parseInt(digits.slice(i, i + 2), 16));
  }
  return bytes;
}

function hexBytes(bytes: readonly number[]): string {
  return bytes
    .map((b) => b.toString(16).padStart(2, "0").toUpperCase())
    .join(" ");
}

const hex16 = (n: number) => hexBytes([n >> 8, n & 0xff]);
const parseHex16 = (text: string) => {
  const [hi, lo] = parseHexBytes(text);
  return (hi << 8) | lo;
};

export const DEFAULT_LORA_FORM: RadioFormValues["lora"] = {
  spreadingFactor: "SF7",
  bandwidth: "Bw125",
  codingRate: "Cr45",
  preambleLength: 8,
  implicitHeader: false,
  payloadLength: 0,
  crcEnabled: true,
  invertIq: false,
  syncWord: "12",
};

export const DEFAULT_GFSK_FORM: RadioFormValues["gfsk"] = {
  bitRate: 9600,
  freqDeviation: 35000,
  pulseShape: "NoFilter",
  preamblePattern: "Alternating01",
  preambleLength: 5,
  syncWord: "2D D4",
  variableLength: true,
  fixedLength: 0,
  crc: "Crc16",
  polynomial: "10 21",
  initial: "00 00",
  invertOutput: false,
  includeLength: true,
  coding: "None",
};

export function radioToForm(radio: RadioConfig): RadioFormValues {
  const base = {
    frequencyMhz: radio.frequency_hz / 1e6,
    lora: DEFAULT_LORA_FORM,
    gfsk: DEFAULT_GFSK_FORM,
  };
  if ("Lora" in radio.phy) {
    const l = radio.phy.Lora;
    const implicit = typeof l.header === "object";
    return {
      ...base,
      modulation: "Lora",
      lora: {
        spreadingFactor: l.spreading_factor,
        bandwidth: l.bandwidth,
        codingRate: l.coding_rate,
        preambleLength: l.preamble_length,
        implicitHeader: implicit,
        payloadLength: typeof l.header === "object" ? l.header.Implicit.payload_length : 0,
        crcEnabled: l.crc_enabled,
        invertIq: l.invert_iq,
        syncWord: hexBytes([l.sync_word]),
      },
    };
  }
  const g = radio.phy.Gfsk;
  const crc = g.crc === "None" ? undefined : g.crc.Crc16;
  return {
    ...base,
    modulation: "Gfsk",
    gfsk: {
      bitRate: g.bit_rate,
      freqDeviation: g.freq_deviation,
      pulseShape: g.pulse_shape,
      preamblePattern: g.preamble_pattern,
      preambleLength: g.preamble_length,
      syncWord: hexBytes(g.sync_word.bytes.slice(0, g.sync_word.len)),
      variableLength: g.length === "Variable",
      fixedLength: typeof g.length === "object" ? g.length.Fixed : 0,
      crc: crc ? "Crc16" : "None",
      polynomial: hex16(crc?.polynomial ?? 0x1021),
      initial: hex16(crc?.initial ?? 0),
      invertOutput: crc?.invert_output ?? false,
      includeLength: crc?.include_length ?? true,
      coding: g.coding,
    },
  };
}

export function formToRadio(form: RadioFormValues): RadioConfig {
  const frequency_hz = Math.round(form.frequencyMhz * 1e6);
  if (form.modulation === "Lora") {
    const l = form.lora;
    const lora: LoraConfig = {
      spreading_factor: l.spreadingFactor,
      bandwidth: l.bandwidth,
      coding_rate: l.codingRate,
      preamble_length: l.preambleLength,
      header: l.implicitHeader
        ? { Implicit: { payload_length: l.payloadLength } }
        : "Explicit",
      crc_enabled: l.crcEnabled,
      invert_iq: l.invertIq,
      sync_word: parseHexBytes(l.syncWord)[0],
    };
    return { frequency_hz, phy: { Lora: lora } };
  }
  const g = form.gfsk;
  const syncBytes = parseHexBytes(g.syncWord);
  const crc: GfskCrc =
    g.crc === "None"
      ? "None"
      : {
          Crc16: {
            polynomial: parseHex16(g.polynomial),
            initial: parseHex16(g.initial),
            invert_output: g.invertOutput,
            include_length: g.includeLength,
          },
        };
  const gfsk: GfskConfig = {
    bit_rate: g.bitRate,
    freq_deviation: g.freqDeviation,
    pulse_shape: g.pulseShape,
    preamble_pattern: g.preamblePattern,
    preamble_length: g.preambleLength,
    sync_word: {
      bytes: Array.from({ length: 8 }, (_, i) => syncBytes[i] ?? 0) as GfskConfig["sync_word"]["bytes"],
      len: syncBytes.length,
    },
    length: g.variableLength ? "Variable" : { Fixed: g.fixedLength },
    crc,
    coding: g.coding,
  };
  return { frequency_hz, phy: { Gfsk: gfsk } };
}
