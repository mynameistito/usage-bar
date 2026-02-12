# Usage Bar for Windows

A system tray application that displays real-time API usage statistics for Claude (Anthropic) and Z.ai (GLM-4.7) services on Windows.

## Prerequisites

- Windows 10 v1809+ or Windows 11
- [Node.js](https://nodejs.org/) (v18 or later)
- [Rust](https://www.rust-lang.org/) (latest stable)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)
- [Bun](https://bun.sh/) (dev)

## Installation

### Install Dependencies

```bash
bun install
```

## Development

Run the application in development mode:

```bash
bun run dev
```

This will launch the application with hot reload enabled.

## Building

### Build All Installers

```bash
bun run build
```

### Build NSIS Installer (Primary)

```bash
bun run build:nsis
```

### Build MSI Installer (Secondary)

```bash
bun run build:msi
```

Built installers will be located in `src-tauri/target/release/bundle/`.

## Usage

### First Run

1. The application will start in the system tray (look for the poo icon)
2. Left-click the tray icon to open the popup panel
3. If Claude Code is installed and authenticated, you should see Claude usage data automatically

### Configuring Z.ai

1. Click "Configure z.ai" in the popup panel
2. Enter your Z.ai API key
3. Click "Save"
4. Z.ai usage data will appear once configured

### Managing the Application

- **Left-click** the tray icon to open/close the popup panel
- **Right-click** the tray icon to show the context menu (Open, Quit)
- Click "Refresh" in the popup to manually refresh usage data
- Click "Quit" to exit the application

## Credential Management

The application reads and stores credentials securely using Windows Credential Manager:

### Claude Credentials

- **Target Name**: `Claude Code-credentials`
- **Source**: Shared with Claude Code application
- **Automatic**: Credentials are read automatically from the same entry used by Claude Code
- **Format**: JSON containing OAuth tokens

### Z.ai API Key

- **Target Name**: `usage-bar-zai-credentials`
- **Source**: Configured through the application UI
- **Manual**: You must enter and save your Z.ai API key through the UI
- **Format**: Plain text API key

## API Contracts

### Claude Usage API

```
GET https://api.anthropic.com/api/oauth/usage
Headers:
  Authorization: Bearer {accessToken}
  anthropic-beta: oauth-2025-04-20
```

### Z.ai Quota API

```
GET https://api.z.ai/api/monitor/usage/quota/limit
Headers:
  Authorization: {apiKey}
  Accept-Language: en-US,en
  Content-Type: application/json
```

## Troubleshooting

### Claude Usage Not Showing

- Make sure Claude Code is installed and authenticated
- Check that credentials exist in Windows Credential Manager (search for "Credential Manager" in Windows)
- Look for an entry named "Claude Code-credentials"
- If missing, authenticate with Claude Code first

### Z.ai API Key Invalid

- Double-check your Z.ai API key for typos
- Make sure the API key is valid and active
- Try deleting and re-entering the API key

### Network Errors

- Check your internet connection
- Verify that API endpoints are accessible
- Try clicking the "Refresh" button manually

### High DPI Display Issues

- The application supports high-DPI scaling (100%, 125%, 150%, 200%, 250%)
- If text appears blurry, check your Windows display settings

## Development Notes

### Project Structure

- `src-tauri/` - Rust backend (credentials, API services, Tauri commands)
- `src/` - Frontend (HTML, TypeScript, CSS)
- `src-tauri/icons/` - Application icons

### Key Technologies

- **Tauri v2** - Desktop application framework
- **Rust** - Backend (credentials, API calls)
- **TypeScript** - Frontend logic
- **Windows Credential Manager** - Secure credential storage
- **Reqwest** - HTTP client

### Building From Source

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js dependencies
bun install

# Run development server
bun run dev

# Build release
bun run build
```

## License

See LICENSE file in the parent directory.

## Support

For issues or questions, please file an issue on the GitHub repository.
