import { useState, useEffect, useRef } from "react";
import { cesiumStore, initSharedViewer } from "./cesium-store";

import * as Cesium from "cesium";
import "cesium/Build/Cesium/Widgets/widgets.css";

const getOffsetPosition = (
  startPosition: Cesium.Cartesian3,
  east: number,
  north: number,
  up: number,
) => {
  const enuToFixed = Cesium.Transforms.eastNorthUpToFixedFrame(startPosition);
  const offsetVector = new Cesium.Cartesian3(east, north, up);
  const result = new Cesium.Cartesian3();

  return Cesium.Matrix4.multiplyByPoint(enuToFixed, offsetVector, result);
};

let trajectoryEntity: Cesium.Entity | null = null;

function updateTrajectory(newPositions: Cesium.Cartesian3[]) {
  if (newPositions.length < 2) return;

  if (!trajectoryEntity || !trajectoryEntity.polyline) {
    if (cesiumStore.viewer)
      trajectoryEntity = cesiumStore.viewer.entities.add({
        name: "Trajectory",
        polyline: {
          positions: newPositions,
          width: 5,
          material: Cesium.Color.RED,
          clampToGround: false,
        },
      });
  } else {
    const polyline = trajectoryEntity.polyline as Cesium.PolylineGraphics;
    polyline.positions = new Cesium.ConstantProperty(newPositions);
  }
}

export default function OnlineMap({
  ionAccessToken,
}: {
  ionAccessToken?: string;
}) {
  const localRef = useRef<HTMLDivElement>(null);

  // Mock trajectory
  const [seconds, setSeconds] = useState(0);
  const [linePositions, setLinePositions] = useState<Cesium.Cartesian3[]>([]);

  useEffect(() => {
    if (!ionAccessToken) return;

    initSharedViewer(ionAccessToken);

    const mapContainer = cesiumStore.container;
    const currentLocalRef = localRef.current;

    if (currentLocalRef && mapContainer) {
      currentLocalRef.appendChild(mapContainer);

      cesiumStore.viewer?.resize();
    }

    if (cesiumStore.viewer) {
      cesiumStore.viewer.scene.globe.enableLighting = false;

      // Build mock positions
      let positions = [];
      const initial_position = Cesium.Cartesian3.fromDegrees(
        149.917516,
        -31.148837,
        330,
      );
      const num_positions = 100;
      for (let i = 0; i < num_positions; i++) {
        positions.push(
          getOffsetPosition(
            initial_position,
            -i * 2,
            i * 1.5,
            (i / 3) * (-i + num_positions - 1),
          ),
        );
      }
      setLinePositions(() => positions);

      const offset = new Cesium.HeadingPitchRange(
        Cesium.Math.toRadians(45),
        Cesium.Math.toRadians(-20),
        150,
      );
      cesiumStore.viewer.camera.lookAt(initial_position, offset);
    }

    return () => {
      if (mapContainer && currentLocalRef?.contains(mapContainer)) {
        currentLocalRef.removeChild(mapContainer);
      }
    };
  }, [ionAccessToken]);

  // Mock trajectory update
  useEffect(() => {
    if (seconds > 100) return;

    const timer = setInterval(() => {
      setSeconds((prevSeconds) => prevSeconds + 1);
      updateTrajectory(linePositions.slice(0, seconds));
    }, 1000);

    return () => clearInterval(timer);
  }, [seconds]);

  return (
    <div className="h-full w-full rounded-xl overflow-hidden border bg-card">
      <div ref={localRef} className="h-full w-full" />
    </div>
  );
}
