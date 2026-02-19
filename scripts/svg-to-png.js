import fs from "node:fs";
import path from "node:path";
import sharp from "sharp";
import toIco from "to-ico";

const svgPath = path.join(process.cwd(), "assets/logo.svg");
const pngOutputPath = path.join(process.cwd(), "src-tauri/icons/icon.png");
const icoOutputPath = path.join(process.cwd(), "src-tauri/icons/icon.ico");

const svg = fs.readFileSync(svgPath);

console.log("Converting SVG to PNG...");

const pngBuffer = await sharp(svg)
  .resize(256, 256, {
    fit: "contain",
    background: { r: 0, g: 0, b: 0, alpha: 0 },
  })
  .png()
  .toBuffer();

fs.writeFileSync(pngOutputPath, pngBuffer);
console.log("✓ PNG generated at:", pngOutputPath);

console.log("Converting PNG to ICO...");

const icoSizes = await Promise.all(
  [16, 32, 48, 256].map((size) =>
    sharp(svg)
      .resize(size, size, {
        fit: "contain",
        background: { r: 0, g: 0, b: 0, alpha: 0 },
      })
      .png()
      .toBuffer()
  )
);

const icoBuffer = await toIco(icoSizes);
fs.writeFileSync(icoOutputPath, icoBuffer);
console.log("✓ ICO generated at:", icoOutputPath);
