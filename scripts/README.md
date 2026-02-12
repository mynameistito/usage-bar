# Scripts

This directory contains build scripts for the Windows version of Usage Bar.

## generate-icon.js

Automatically converts `src-tauri/icons/icon.png` to `src-tauri/icons/icon.ico` during the build process.

### Usage

The script runs automatically as part of the build process:

```bash
bun run build
```

Or run it manually:

```bash
bun run generate-icon
```

### Dependencies

- `png-to-ico` - Converts PNG to ICO format
- `sharp` (optional) - Creates multiple resolution versions for better scaling

If `sharp` is not installed, the script will create a single-resolution icon. To install sharp:

```bash
bun add --dev sharp
```

### Manual Conversion

If the automated script fails, you can manually convert the icon using:

#### Online Tools
- https://convertio.co/png-ico/
- https://cloudconvert.com/png-to-ico

#### Command Line (ImageMagick)
```bash
magick convert icon.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico
```

#### Node.js
```bash
bunx png-to-ico icon.png > icon.ico
```
