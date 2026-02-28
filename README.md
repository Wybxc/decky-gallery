# decky-gallery

A simple web server that displays your Steam screenshots as a browsable gallery. It scans the Steam userdata directory at `~/.local/share/Steam/userdata` for screenshots, sorts them by modification time with the most recent first, and serves them as a responsive image grid on a local web page.

## Building

You need a Rust toolchain installed. Clone the repository and run `cargo build --release`. The compiled binary will be placed at `target/release/decky-gallery`.

## Running

Run the binary directly. By default it listens on port 3000, so opening `http://localhost:3000` in a browser will show your screenshots. You can change the port with the `-p` flag, for example `decky-gallery -p 8080`.

## Running as a service

A systemd user service file is included. Copy `decky-gallery.service` to `~/.config/systemd/user/`, then enable and start it with `systemctl --user enable --now decky-gallery`. The service expects the binary to be installed at `~/.cargo/bin/decky-gallery`, which is the default location when you run `cargo install --path .`.
