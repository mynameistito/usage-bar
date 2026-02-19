import fs from "node:fs";
import path from "node:path";
import sharp from "sharp";

const svgPath = path.join(process.cwd(), "assets/logo.svg");
const pngOutputPath = path.join(process.cwd(), "src-tauri/icons/icon.png");
const icoOutputPath = path.join(process.cwd(), "src-tauri/icons/icon.ico");

const svg = fs.readFileSync(svgPath);

/**
 * Encodes an array of PNG buffers into an ICO file buffer.
 * Embeds PNGs directly (valid for Windows Vista+ / 256px images).
 * @param {Array<{ png: Buffer, size: number }>} images
 * @returns {Buffer}
 */
function encodeIco(images) {
  const ICO_HEADER_SIZE = 6;
  const DIRECTORY_ENTRY_SIZE = 16;
  const directorySize = images.length * DIRECTORY_ENTRY_SIZE;

  const header = Buffer.alloc(ICO_HEADER_SIZE);
  header.writeUInt16LE(0, 0); // reserved
  header.writeUInt16LE(1, 2); // type: 1 = ICO
  header.writeUInt16LE(images.length, 4);

  const entries = [];
  let offset = ICO_HEADER_SIZE + directorySize;

  for (const { png, size } of images) {
    const entry = Buffer.alloc(DIRECTORY_ENTRY_SIZE);
    // Width/height: 0 encodes as 256 per the ICO spec
    entry.writeUInt8(size >= 256 ? 0 : size, 0);
    entry.writeUInt8(size >= 256 ? 0 : size, 1);
    entry.writeUInt8(0, 2); // color count (0 = no palette)
    entry.writeUInt8(0, 3); // reserved
    entry.writeUInt16LE(1, 4); // color planes
    entry.writeUInt16LE(32, 6); // bits per pixel
    entry.writeUInt32LE(png.length, 8);
    entry.writeUInt32LE(offset, 12);
    entries.push(entry);
    offset += png.length;
  }

  return Buffer.concat([header, ...entries, ...images.map(({ png }) => png)]);
}

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

const ICO_SIZES = [16, 32, 48, 256];

const icoImages = await Promise.all(
  ICO_SIZES.map(async (size) => {
    const png = await sharp(svg)
      .resize(size, size, {
        fit: "contain",
        background: { r: 0, g: 0, b: 0, alpha: 0 },
      })
      .png()
      .toBuffer();
    return { png, size };
  })
);

const icoBuffer = encodeIco(icoImages);
fs.writeFileSync(icoOutputPath, icoBuffer);
console.log("✓ ICO generated at:", icoOutputPath);
