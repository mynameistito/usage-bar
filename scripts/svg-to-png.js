import fs from "node:fs";
import path from "node:path";
import sharp from "sharp";

const svgPath = path.join(process.cwd(), "assets/logo.svg");
const outputPath = path.join(process.cwd(), "src-tauri/icons/icon.png");

console.log("Converting SVG to PNG...");

const svg = fs.readFileSync(svgPath);

sharp(svg)
  .resize(256, 256, {
    fit: "contain",
    background: { r: 0, g: 0, b: 0, alpha: 0 },
  })
  .png()
  .toFile(outputPath)
  .then(() => {
    console.log("✓ PNG generated at:", outputPath);
  })
  .catch((err) => {
    console.error("✗ Error:", err.message);
    process.exit(1);
  });
