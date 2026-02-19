import fs from "node:fs";
import path from "node:path";
import pngToIco from "png-to-ico";

// Try to import sharp, but handle if it's not installed
let sharp;
try {
  sharp = (await import("sharp")).default;
} catch {
  sharp = null;
}

const sourcePng = path.join(process.cwd(), "src-tauri/icons/icon.png");
const outputIco = path.join(process.cwd(), "src-tauri/icons/icon.ico");

console.log("Converting icon.png to icon.ico...");

const sizes = [16, 20, 24, 32, 48, 64];
const pngBuffers = [];

async function createScaledPng(size) {
  if (!sharp) {
    throw new Error("sharp not installed");
  }
  return await sharp(sourcePng)
    .resize(size, size, { fit: "cover" })
    .png()
    .toBuffer();
}

async function generateIcon() {
  try {
    if (sharp) {
      console.log("Creating multiple resolutions with sharp...");
      for (const size of sizes) {
        const buffer = await createScaledPng(size);
        pngBuffers.push(buffer);
      }

      const icoBuffer = await pngToIco(pngBuffers);
      fs.writeFileSync(outputIco, icoBuffer);
      console.log("✓ Icon generated successfully at:", outputIco);
    } else {
      console.log("sharp not installed, creating single-resolution icon...");
      const sourceBuffer = fs.readFileSync(sourcePng);
      const icoBuffer = await pngToIco(sourceBuffer);
      fs.writeFileSync(outputIco, icoBuffer);
      console.log("✓ Icon generated successfully at:", outputIco);
      console.log(
        "  (Single resolution - install sharp for multi-resolution support)"
      );
    }
  } catch (error) {
    console.error("✗ Error generating icon:", error.message);
    console.log("");
    console.log(
      "If this fails, you can manually convert icon.png to icon.ico using:"
    );
    console.log("1. Online tools: https://convertio.co/png-ico/");
    console.log(
      "2. ImageMagick: magick convert icon.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico"
    );
    process.exit(1);
  }
}

generateIcon();
