import fs from 'fs';
import path from 'path';
import pngToIco from 'png-to-ico';
import sharp from 'sharp';

const sourcePng = path.join(process.cwd(), 'src-tauri/icons/icon.png');
const outputIco = path.join(process.cwd(), 'src-tauri/icons/icon.ico');

console.log('Converting icon.png to icon.ico...');

const sizes = [16, 20, 24, 32, 48, 64];
const pngBuffers = [];

async function createScaledPng(size) {
  return await sharp(sourcePng)
    .resize(size, size, { fit: 'cover' })
    .png()
    .toBuffer();
}

async function generateIcon() {
  try {
    console.log('Creating multiple resolutions...');
    for (const size of sizes) {
      const buffer = await createScaledPng(size);
      pngBuffers.push(buffer);
    }

    const icoBuffer = await pngToIco(pngBuffers);
    fs.writeFileSync(outputIco, icoBuffer);
    console.log('✓ Icon generated successfully at:', outputIco);
  } catch (error) {
    console.error('✗ Error generating icon:', error.message);
    console.log('');
    console.log('If this fails, you can manually convert icon.png to icon.ico using:');
    console.log('1. Online tools: https://convertio.co/png-ico/');
    console.log('2. ImageMagick: magick convert icon.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico');
  }
}

generateIcon();
