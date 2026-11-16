import * as Cesium from "cesium";

interface CesiumStore {
  viewer: Cesium.Viewer | null;
  container: HTMLDivElement | null;
}

export const cesiumStore: CesiumStore = {
  viewer: null,
  container: null,
};

export const initSharedViewer = (accessToken: string) => {
  if (cesiumStore.viewer) return;

  Cesium.Ion.defaultAccessToken = accessToken;

  const container = document.createElement("div");
  container.style.width = "100%";
  container.style.height = "100%";

  cesiumStore.viewer = new Cesium.Viewer(container, {
    terrain: Cesium.Terrain.fromWorldTerrain(),
    timeline: false, // Removes the timeline control
    animation: false, // Removes the animation control
    geocoder: false, // Removes the search button (top right)
    homeButton: false, // Removes the home button
    sceneModePicker: false, // Removes the 2D/3D switcher
    baseLayerPicker: false, // Removes the map layer selector
    navigationHelpButton: false, // Removes the question mark button
    infoBox: false, // Removes the info pop-up when clicking entities
    selectionIndicator: false, // Removes the green selection box
    fullscreenButton: false, // Removes the fullscreen button (bottom right)
  });
  cesiumStore.container = container;
};
