# Networkless-RS

A simple network manager applet for Linux.

## Requirements

- `networkmanager`

## Installation

1. Build the program:

```bash
cargo build --release
```

2. Move the executable to bin folder in your PATH:

```bash
mv target/release/networkless-rs ~/.local/bin
```

## Acknowledgements

This borrows a lot from:

- [gnome-shell network applet](https://github.com/GNOME/gnome-shell/blob/765c4b622b9168057304a2cc491c5ba6a41f7439/js/ui/status/network.js)
- [cosmic-applet-network](https://github.com/pop-os/cosmic-applets/tree/c171f048a6dff1a032eb5edf8f343cac60971ac5/cosmic-applet-network)

I already like how gnome does things, so I wanted to do something similar.
