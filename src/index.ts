import { AniSS } from "../third_party/ani-ss/pkg";
import { SHADER } from "./shader";

// Direct link to video (.m3u8 stream since that's what most anime sites use and we're gonna be scraping them)
const VIDEO_SRC: string =
    "https://v.vrv.co/evs1/4b04d7820cb37b4b9e901558ba1b17d9/assets/7620b90db071e6d7b62287ca158fe2c1_,4270109.mp4,4270117.mp4,4270101.mp4,4270085.mp4,4270093.mp4,.urlset/master.m3u8?Policy=eyJTdGF0ZW1lbnQiOlt7IlJlc291cmNlIjoiaHR0cCo6Ly92LnZydi5jby9ldnMxLzRiMDRkNzgyMGNiMzdiNGI5ZTkwMTU1OGJhMWIxN2Q5L2Fzc2V0cy83NjIwYjkwZGIwNzFlNmQ3YjYyMjg3Y2ExNThmZTJjMV8sNDI3MDEwOS5tcDQsNDI3MDExNy5tcDQsNDI3MDEwMS5tcDQsNDI3MDA4NS5tcDQsNDI3MDA5My5tcDQsLnVybHNldC9tYXN0ZXIubTN1OCIsIkNvbmRpdGlvbiI6eyJEYXRlTGVzc1RoYW4iOnsiQVdTOkVwb2NoVGltZSI6MTY0MzA1Nzc1MX19fV19&Signature=HkuRwOIygaASoQ40BdDXpjhCeIuHeucyRN0t2OwEYfR3ZvWefWheVpaZnHc~JS9SGsUeK6wHQh9WB7rwtS6lfg74c~e8NffSZLFXz-LEZM0rwO9Sqtli97QIY7u4AF0FqCijMz15xgOF6qHPeuO7MENEGxLZxpKuo445mqpSFpOoAK3fC051wdKVQRMrhoe8O9fDiMrAMPqi0ThEomDG7PlS6LqlGoEOm~OMe0Yx~u9YFcV1FVC31CwBDMNxvEYDqh1t-Fc66QfE8QDhECRKMC8Var7DoGnDnHV7vGg1AKOhIjGNdIHP~ZnX9ooqry0YZcnUSMLL-wm7xHtrYuc~QA__&Key-Pair-Id=APKAJMWSQ5S7ZB3MF5VA";

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
