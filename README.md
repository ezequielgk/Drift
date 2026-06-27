# drift

*drift* is a Sway WM tool that manages a horizontal scroll-style workspace layout via the Sway IPC protocol.

It follows the Unix philosophy: it does one thing and does it well. *drift* never reads, writes, or manages any keybindings. Instead, it exposes a set of discrete CLI actions that map directly to Sway workspace commands. You bind those actions to keys in your own Sway config. 

When *drift* is inactive, all action commands exit immediately without doing anything. When active, they execute the corresponding Sway IPC command to navigate the layout.

```
        BINARY       IPC SOCKET
drift  ------------------------>  sway
```

## Variants

### `drift` — stateless

No autostart required. State is persisted in `/tmp/drift.lock`.

```
drift toggle       # toggle the layout on/off
drift activate     # force activate
drift deactivate   # force deactivate
drift status       # print "active" or "inactive"

# Configuration
drift config get max-windows
drift config set max-windows 3

# Window overflow
drift overflow     # manually check window count and move to next workspace if exceeded
```

### `driftd` — daemon

Starts at Sway session launch. Holds state in memory. Accepts commands over `/tmp/drift.sock`.

```
driftd             # start the daemon
drift-ctl toggle
drift-ctl activate
drift-ctl deactivate
drift-ctl status

# Configuration
drift-ctl config get max-windows
drift-ctl config set max-windows 3
```

## Available Actions

Both `drift` and `drift-ctl` provide the following actions:

| Action       | What it does when active                                  |
|--------------|-----------------------------------------------------------|
| `next`       | focus next workspace on output                            |
| `prev`       | focus previous workspace on output                        |
| `move-next`  | move container to next workspace and follow               |
| `move-prev`  | move container to prev workspace and follow               |
| `back`       | toggle between last two workspaces                        |
| `overflow`   | *(stateless only)* manually check limit and trigger overflow |

*(When inactive, these commands do nothing and exit 0).*

## Window Overflow & Config

*drift* supports an automatic **window overflow** feature. You can define a maximum number of windows per workspace. If *drift* is active and a new window is opened that exceeds this limit, the window is automatically moved to the next workspace on the output.

The configuration is saved persistently in `~/.config/drift/config.toml`:

```toml
max_windows = 2
```

- **Daemon variant (`driftd`)**: This feature works automatically in the background. The daemon listens to Sway window events and triggers the overflow without any extra configuration.
- **Stateless variant (`drift`)**: Since it doesn't run in the background, you must manually trigger the overflow check in your Sway config using the `for_window` directive.

## Installation

```bash
git clone https://github.com/ezequielgk/drift
cd drift
cargo install --path crates/drift    # stateless variant
cargo install --path crates/driftd   # daemon variant
```

## Sway integration

Bind your chosen keys directly to the `drift` actions in your Sway configuration.

### Stateless

```bash
# ~/.config/sway/config

# Toggle drift on/off
bindsym $mod+F1 exec drift toggle

# Bind drift actions
bindsym $mod+Right       exec drift next
bindsym $mod+Left        exec drift prev
bindsym $mod+Shift+Right exec drift move-next
bindsym $mod+Shift+Left  exec drift move-prev
bindsym $mod+Tab         exec drift back

# Enable window overflow (Stateless only)
for_window [app_id=".*"] exec drift overflow
for_window [class=".*"] exec drift overflow
```

### Daemon

```bash
# ~/.config/sway/config

# Start the daemon
exec driftd

# Toggle drift on/off
bindsym $mod+F1 exec drift-ctl toggle

# Bind drift actions
bindsym $mod+Right       exec drift-ctl next
bindsym $mod+Left        exec drift-ctl prev
bindsym $mod+Shift+Right exec drift-ctl move-next
bindsym $mod+Shift+Left  exec drift-ctl move-prev
bindsym $mod+Tab         exec drift-ctl back
```

## Options

```
--socket <PATH>    override $SWAYSOCK
```

## License

GPL-3.0-only