# Clammy


https://github.com/user-attachments/assets/dd026482-48a2-48a3-9e8e-ecca2be73ee3

## Prerequisites
- Rust
- Cargo
- Git

## Quickstart
- Clone the repository (`git clone https://github.com/spinualexandru/clammy`)
- Navigate to the cloned directory (`cd clammy`)
- Run `cargo install --path .`
- Run `clammy` in the terminal

## Features

### General
- Sync colors from Matugen
- Hotreload config

### Widgets
- Clock
- Battery
- Window title
- Workspaces

## Configuration

Configuration should be placed in the `~/.config/clammy/config.toml` file.

Defaults for the color scheme is a tokyo night inspired color scheme.

Matugen example can be found in the `./docs/config.toml` file.

```toml
[theme]

# Font
font = "BlexMono Nerd Font Mono" # Default is monospace
font_size = 16 # Default is 16

# Spacings
tray_widget_spacing = 0 # Default is 8
tray_widget_padding = 4 # Default is 8

# Core palette
background = "#f5fafe"
background_alpha = 0.85
text = "#171c1f"
success = "#8b5000"
danger = "#ba1a1a"

# Extended colors
accent = "#006686"
accent2 = "#3d6376"
info = "#8b5000"
surface = "#eaeef2"
surface_alpha = 0.94
border = "#6e797f"
muted = "#bdc8cf"
hover = "#e4e9ed"
hover_alpha = 0.5
```

## Roadmap

### Widgets
- [x] Clock
- [x] Battery
- [x] Window title
- [x] Workspaces

#### Popups

- [ ] Make tray popup drop down with an animation
- [ ] Make tray popup seem like it's connecting with the bar

#### Workspaces
- [x] Make the workspace change look animated

#### Clock
- [ ] Dropdown Clock

### General
- [x] Sync colors from Matugen

#### Performance
- [x] Improve memory usage

#### Nice to have
- [ ] Sync the border of the status bar with the laptop
