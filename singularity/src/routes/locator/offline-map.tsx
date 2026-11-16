import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const NUM_RINGS = 3;
const MAX_RANGE_M = 1000;

interface Bearing {
  heading: number;
  distance: number;
}

function drawRadar(ctx: CanvasRenderingContext2D, bearing: Bearing) {
  if (!ctx) return;

  const half = Math.min(ctx.canvas.width / 2, ctx.canvas.height / 2);

  // 1. Clear the canvas
  ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);

  // --- Grid Setup (Rings and Crosshairs) ---
  ctx.strokeStyle = "rgba(0, 0, 0, 0.4)";
  ctx.lineWidth = 2;

  const ringPadding = 24;
  // Draw Range Rings
  for (let i = 1; i <= NUM_RINGS; i++) {
    const ringRadius = (half / NUM_RINGS) * i;
    ctx.beginPath();
    ctx.arc(half, half, ringRadius - ringPadding, 0, Math.PI * 2);
    ctx.stroke();
  }

  // Draw North, South, East, West lines
  ctx.beginPath();
  ctx.moveTo(half, 0);
  ctx.lineTo(half, ctx.canvas.height); // North-South
  ctx.moveTo(0, half);
  ctx.lineTo(ctx.canvas.width, half); // East-West
  ctx.strokeStyle = "rgba(0, 0, 0, 1)";
  ctx.stroke();

  // Draw Cardinal Direction Labels
  ctx.fillStyle = "#000000";
  ctx.font = "16px Inter";
  ctx.textAlign = "center";
  ctx.fillText("N", half + ringPadding / 2, 18);
  ctx.fillText("S", half + ringPadding / 2, ctx.canvas.height - 5);
  ctx.fillText("E", ctx.canvas.width - ringPadding / 2, half - ringPadding / 2);
  ctx.fillText("W", ringPadding / 2, half - ringPadding / 2);

  const targetX =
    half +
    ((Math.sin((bearing.heading * Math.PI) / 180) * bearing.distance) /
      MAX_RANGE_M) *
      half;
  const targetY =
    half -
    ((Math.cos((bearing.heading * Math.PI) / 180) * bearing.distance) /
      MAX_RANGE_M) *
      half;

  if (bearing.distance <= MAX_RANGE_M) {
    // Draw the Sweep Line (from center to target)
    ctx.strokeStyle = "rgba(255, 65, 0, 0.8)";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(half, half);
    ctx.lineTo(targetX, targetY);
    ctx.stroke();

    // Draw the Target Dot
    ctx.fillStyle = "#ff4100"; // Red for target
    ctx.beginPath();
    ctx.arc(targetX, targetY, 5, 0, Math.PI * 2);
    ctx.fill();
  }
}

export default function OfflineMap() {
  const [_seconds, _setSeconds] = useState(0);
  const [_state, _setState] = useState(null);

  // const [linePositions, setLinePositions] = useState<Cesium.Cartesian3[]>([]);

  useEffect(() => {
    const canvas = document.getElementById("radar-canvas") as HTMLCanvasElement;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (ctx) {
      drawRadar(ctx, { heading: 312, distance: 645 });
    }
  }, []);

  listen<string>("odometry_updated", () => {
    invoke("get_odometry").then((_odo) => {});
  });

  return (
    <div className="card card-bordered card-compact shadow-md max-h-full overflow-scroll">
      <div className="card-body flex flex-row flex-wrap justify-center">
        <div className="lg:col-span-2 justify-center items-center p-2 rounded-xl info-card max-w-xl max-h-xl aspect-square">
          <canvas
            id="radar-canvas"
            className="max-w-full max-h-full"
            width="1000"
            height="1000"
          ></canvas>
        </div>
        <div className="card card-bordered card-compact shadow-md max-h-full grow">
          <div className="card-body">skjdbfksjdfvkjsbdkfbsdkljf</div>
        </div>
      </div>
    </div>
  );
}
