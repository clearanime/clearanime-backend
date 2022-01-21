import { AniSS } from "../third_party/ani-ss/pkg";
import { SHADER } from "./shader";

// Direct link to video (.m3u8 stream since that's what most anime sites use and we're gonna be scraping them)
const VIDEO_SRC: string =
    "https://v.vrv.co/evs1/e3ae5cc528b9eabb6c69aa8ec7c26e44/assets/e603ae3d7bf23668a51248d0f1e453b9_,4261758.mp4,4261763.mp4,4261753.mp4,4261743.mp4,4261748.mp4,.urlset/master.m3u8?Policy=eyJTdGF0ZW1lbnQiOlt7IlJlc291cmNlIjoiaHR0cCo6Ly92LnZydi5jby9ldnMxL2UzYWU1Y2M1MjhiOWVhYmI2YzY5YWE4ZWM3YzI2ZTQ0L2Fzc2V0cy9lNjAzYWUzZDdiZjIzNjY4YTUxMjQ4ZDBmMWU0NTNiOV8sNDI2MTc1OC5tcDQsNDI2MTc2My5tcDQsNDI2MTc1My5tcDQsNDI2MTc0My5tcDQsNDI2MTc0OC5tcDQsLnVybHNldC9tYXN0ZXIubTN1OCIsIkNvbmRpdGlvbiI6eyJEYXRlTGVzc1RoYW4iOnsiQVdTOkVwb2NoVGltZSI6MTY0MjkyNjU4Nn19fV19&Signature=o6dHPNiHlMf2oS8gk8vGM70o2ScTlDTVariWHrWhj85fmOiIyQ9Njx4CaArL0yQYr7-h-WPoYwdu2LAKb6ooTb~OIrO6uRXlwX1QORRoi7PgYO~5irt2y6WZJRboNceXme9wA~ojBC4lBQDXWjyUIFPzUYGnfXLWMX3eB06WQyH2mthEjcACQ5cW6yznsed69QWwNRlgTsBpdjaNkykm5Xsxg-NnYoiS1jVFQ8MqVhhwY8yVXE3Anq7MxWKKfG2sl6qOvIYdAZXNnHlGVMReipa5mhAftS0X~gzU6hf2ulKR0h8~tjTmD~DpD6T02Rhfox34OzYzGIZNgS04Y47UGQ__&Key-Pair-Id=APKAJMWSQ5S7ZB3MF5VA";

const canvas = document.getElementById("canvas") as HTMLCanvasElement;
const gl = canvas.getContext("webgl2");

// Video tag (not actually visible, it's the canvas that shows the video)
const videoElement = document.createElement("video");
const aniSS = new AniSS(gl);

videoElement.addEventListener("canplay", setup_anime4k); // Setup anime upscaling module only when video dimensions are known
videoElement.crossOrigin = "anonymous"; // To prevent a tainted canvas

import("hls.js")
    .then(({ default: Hls }) => {
        var hls = new Hls();
        hls.loadSource(VIDEO_SRC);
        hls.attachMedia(videoElement);

        hls.on(Hls.Events.LEVEL_LOADED, () => {
            hls.currentLevel = hls.levels.length - 1;
        });

        document.getElementById("play").addEventListener("click", () => {
            videoElement.play();
        });
    })
    .catch((e) => {
        console.error(`Failed to import hls.js: ${e}`);
    });

let anime_upscaling_loaded = false;

function setup_anime4k() {
    if (anime_upscaling_loaded) {
        return;
    } else {
        anime_upscaling_loaded = true;
    }

    // Setup anime upscaling
    aniSS.setSource(videoElement);
    aniSS.addProgram(SHADER);
    aniSS.setScale(1);

    // Start rendering from the video tag to the canvas

    function render() {
        aniSS.render();
        requestAnimationFrame(render);
    }

    requestAnimationFrame(render);
}
