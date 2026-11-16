import type { Capabilities, CapabilityKey } from "@/context/settings/Device";
import type { ComponentType } from "react";
import { DeploymentCard } from "./deployment-card";
import { RadioCard } from "./radio-card";

interface CardProps<K extends CapabilityKey> {
  value: Capabilities[K];
  onChange: (value: Capabilities[K]) => void;
}

/**
 * The card for each capability. Adding a capability means one entry here
 * (the mapped type fails to compile until there is one) and a field on
 * `Capabilities`; the Configuration page needs no changes.
 */
const CARDS: { [K in CapabilityKey]: ComponentType<CardProps<K>> } = {
  radio: RadioCard,
  deployment: DeploymentCard,
};

export function CapabilityCard<K extends CapabilityKey>({
  capability,
  ...props
}: CardProps<K> & { capability: K }) {
  const Card = CARDS[capability] as ComponentType<CardProps<K>>;
  return <Card {...props} />;
}
