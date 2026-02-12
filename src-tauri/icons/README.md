# Icons

The `icon.png` file is the source icon. It needs to be converted to `icon.ico` format with multiple resolutions for DPI scaling.

## Required Resolutions

The `.ico` file should include these resolutions for proper DPI scaling on Windows:
- 16x16 (100% DPI)
- 20x20 (125% DPI)
- 24x24 (150% DPI)
- 32x32 (200% DPI)
- 48x48 (250% DPI)
- 64x64 (extra scaling)

## Conversion Tools

### Online Tools
- https://convertio.co/png-ico/
- https://cloudconvert.com/png-to-ico
- https://www.icoconverter.com/

### Command Line Tools
- ImageMagick: `magick convert icon.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico`
- GIMP (with ICO plugin)

### Node.js Packages
- `png-to-ico`: https://www.npmjs.com/package/png-to-ico

## Quick Conversion

After converting `icon.png` to `icon.ico`, the file should be named `icon.ico` and placed in this directory.

## Note

The `.ico` file is required for the Windows build to work. Without it, the application will fail to build or run.
