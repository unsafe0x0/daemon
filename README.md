# CoreDump Activity Daemon

A standalone daemon that monitors your Zed editor activity and tracks coding time per language, sending it to coredump.vercel.app.

## Features

- Monitors when Zed is the active window
- Tracks coding time per programming language
- Automatically sends activity data to CoreDump API
- Runs in the background as a systemd service
- Respects idle time (stops tracking after 60 seconds of inactivity)

## Requirements

- **Linux** (uses X11 tools for window monitoring)
- **xdotool** - for detecting active windows
- **Rust/Cargo** - for building the daemon
- **systemd** - for running as a background service

## Installation

### 1. Install xdotool

```bash
sudo apt-get install xdotool
```

Or on other distros:

```bash
# Fedora
sudo dnf install xdotool

# Arch
sudo pacman -S xdotool
```

### 2. Run the installation script

```bash
./install.sh
```

The script will:

- Check for required dependencies
- Build the daemon
- Prompt for your CoreDump private key
- Install the binary to `~/.cargo/bin/`
- Set up and start the systemd service

## Configuration

The configuration file is located at `~/.config/coredump/config.toml`:

```toml
private_key = "your-private-key-here"
```

You can edit this file at any time to change your private key, then restart the daemon:

```bash
systemctl --user restart coredump-daemon
```

## Usage

Once installed, the daemon runs automatically in the background. It will:

1. Check every 5 seconds if Zed is the active window
2. Track the current file and detect its language
3. Accumulate time per language
4. Send activity data every 30 seconds (if >= 30 seconds of activity)

### Managing the Service

**Check status:**

```bash
systemctl --user status coredump-daemon
```

**View live logs:**

```bash
journalctl --user -u coredump-daemon -f
```

**Stop the daemon:**

```bash
systemctl --user stop coredump-daemon
```

**Start the daemon:**

```bash
systemctl --user start coredump-daemon
```

**Restart the daemon:**

```bash
systemctl --user restart coredump-daemon
```

**Disable auto-start:**

```bash
systemctl --user disable coredump-daemon
```

## How It Works

1. **Window Detection**: Uses `xdotool` to check if Zed is the active window
2. **File Detection**: Extracts the current filename from the window title
3. **Language Detection**: Maps file extensions to programming languages
4. **Activity Tracking**: Records time spent in each language
5. **Idle Detection**: Stops counting after 60 seconds of no window focus changes
6. **API Submission**: Sends accumulated time to the CoreDump API every 30 seconds

## Supported Languages

The daemon detects 40+ languages including:

- Rust, JavaScript, TypeScript, Python, Go
- Java, C, C++, C#, PHP, Ruby, Swift, Kotlin
- HTML, CSS, SCSS, JSON, YAML, TOML, XML
- Shell, Lua, SQL, Markdown, and many more

Files with unknown extensions are tracked as "Unknown".

## Troubleshooting

### Daemon not starting

Check the logs:

```bash
journalctl --user -u coredump-daemon -n 50
```

### xdotool not working

Make sure you're running an X11 session (not Wayland):

```bash
echo $XDG_SESSION_TYPE
```

If using Wayland, you'll need to switch to X11 or modify the daemon to use Wayland tools.

### Not tracking activity

1. Verify Zed is running and active
2. Check that the window title shows a filename
3. Ensure your private key is correct in the config file

## Uninstallation

```bash
# Stop and disable the service
systemctl --user stop coredump-daemon
systemctl --user disable coredump-daemon

# Remove the service file
rm ~/.config/systemd/user/coredump-daemon.service

# Remove the binary
cargo uninstall coredump

# Remove config (optional)
rm -rf ~/.config/coredump
```

## Development

### Building

```bash
cargo build --release
```

### Running manually (for testing)

```bash
cargo run
```

### Project Structure

```
cor/
├── src/
│   ├── main.rs           # Main daemon logic
│   └── lib.rs            # Zed extension stub (not used)
├── Cargo.toml            # Rust dependencies
├── install.sh            # Installation script
├── coredump-daemon.service  # Systemd service file
└── README.md             # This file
```

## License

This project is part of the CoreDump ecosystem.

## Support

For issues or feature requests, please contact the CoreDump team or check the API documentation at coredump.vercel.app.
