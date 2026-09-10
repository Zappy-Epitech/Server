# Zappy — Server

Server implementation for the Epitech **Zappy** project (`G-YEP-400`), written in Rust.

Zappy simulates the planet **Trantor**, a toroidal world where several teams of drones
(AI clients) gather resources, reproduce and perform elevation rituals. The first team
to get **6 players to level 8** wins.

This repository holds only the server. It is consumed as a submodule by
[`Zappy-Epitech/Common`](https://github.com/Zappy-Epitech/Common), which bundles the
server, the AI and the GUI.

---

## Table of contents

- [Build](#build)
- [Usage](#usage)
- [Architecture](#architecture)
- [Game rules implemented](#game-rules-implemented)
- [AI protocol](#ai-protocol)
- [GUI protocol](#gui-protocol)
- [Bonus: TUI dashboard](#bonus-tui-dashboard)
- [Tests](#tests)
- [Implementation notes](#implementation-notes)

---

## Build

Requires a Rust toolchain with **edition 2024** support (developed against `rustc 1.98`).

```bash
cargo build --release      # binary: target/release/zappy_server
cargo build                # debug build
cargo run -- -p 4242 -x 20 -y 20 -n team1 team2 -c 5 -f 100
```

Dependencies (`Cargo.toml`):

| Crate | Role |
| --- | --- |
| `clap` (derive) | Command-line argument parsing |
| `mio` | Non-blocking TCP + event polling (single-threaded reactor) |
| `rand` | Resource spawning, spawn positions, initial orientation |
| `ratatui` / `crossterm` | Terminal dashboard (bonus, `-b`) |

## Usage

```
USAGE: ./zappy_server -p port -x width -y height -n name1 name2 ... -c clientsNb -f freq [-b]
```

| Flag | Type | Description |
| --- | --- | --- |
| `-p` | `u16` | Port the server listens on (binds `0.0.0.0`) |
| `-x` | `u32` | Width of the world |
| `-y` | `u32` | Height of the world |
| `-n` | `String...` | Team names (one or more) |
| `-c` | `usize` | Number of authorized clients per team at start |
| `-f` | `u32` | Reciprocal of the time unit — defaults to `100` |
| `-b`, `--bonus` | flag | Run the Ratatui dashboard instead of plain stdout logs |

One **time unit** lasts `1 / f` seconds, so a higher `-f` means a faster game.

Example:

```bash
./zappy_server -p 4242 -x 20 -y 20 -n red blue -c 6 -f 100
```

## Architecture

```
src/
├── main.rs                    entrypoint — parses config, starts server (+ TUI thread if -b)
├── config.rs                  ServerConfig, clap CLI definition
│
├── network/
│   ├── server.rs              mio event loop, client registry, game tick, dispatching
│   └── client.rs              Client (socket + in/out buffers) and ClientState
│
├── protocol/
│   ├── mod.rs                 Command enum for AI clients + durations + parsing
│   └── gui.rs                 GuiCommand enum + parsing
│
├── game/
│   ├── world.rs               World, Tile, Egg, Resource, spawning, victory check
│   ├── player.rs              Player, Direction, time-based food model
│   └── commands/
│       ├── mod.rs             init/execute dispatch + compute_direction (sound bearing)
│       ├── movement.rs        Forward, Right, Left, Eject
│       ├── interaction.rs     Look, Inventory, Take, Set, Fork
│       ├── social.rs          Broadcast
│       └── incantation.rs     Elevation ritual (requirement table, start/end checks)
│
├── gui/commands/
│   ├── map.rs                 msz, bct, mct, tna
│   ├── player.rs              ppo, plv, pin
│   └── time.rs                sgt, sst (rescales every pending deadline)
│
└── tui/
    ├── mod.rs                 ServerEvent enum (server → TUI channel)
    └── app.rs                 Ratatui dashboard rendering + input handling
```

### Event loop

The server is **single-threaded and fully non-blocking**. `mio::Poll` watches the
listener (`Token(0)`) and every client socket; each iteration:

1. Compute a timeout equal to the closest upcoming deadline — next command completion,
   next player death, or next resource respawn (capped at 100 ms).
2. Poll for readable/writable events, accept new connections, read into `buffer_in`,
   flush `buffer_out`.
3. Split `buffer_in` on `\n` and process each complete line according to the client's state.
4. `update_game()` — respawn resources, kill starving players, execute every command
   whose `end_time` has elapsed, drain `world.gui_events` to graphic clients, check victory.
5. `flush_notifications()` — push asynchronous messages (`message`, `eject`) to AI clients.

### Client state machine

`ClientState` in `network/client.rs`:

| State | Meaning |
| --- | --- |
| `Authenticating` | Connected, `WELCOME` sent, waiting for a team name or `GRAPHIC` |
| `InGame(id)` | AI client bound to the player with that id |
| `Graphic` | Graphic client, receives every world event |

An unknown or full team answers `ko` and leaves the client in `Authenticating`, so it may retry.

### Command scheduling

AI commands are **queued, not executed on arrival**. On receipt the server:

- runs `commands::init()` for commands that need an immediate reply
  (`Incantation` → `Elevation underway`/`ko`, `Fork` → `pfk` to the GUI);
- refuses the command silently if the queue already holds **10** entries;
- otherwise computes `end_time = max(last_command_end, now) + duration / freq` and
  pushes it, so queued commands run back-to-back rather than in parallel.

`update_game()` pops and executes them once their `end_time` is reached.

## Game rules implemented

### The world

- The map is a **torus**: every movement, `Look` cone and sound bearing wraps around.
- Tiles hold resource counts in a `HashMap<Resource, u32>`.

### Resources

Every **20 time units** the server tops each resource back up to
`ceil(width * height * density)` across the whole map, scattering the missing units at
random positions:

| Resource | Density |
| --- | --- |
| food | 0.5 |
| linemate | 0.3 |
| deraumere | 0.15 |
| sibur | 0.1 |
| mendiane | 0.1 |
| phiras | 0.08 |
| thystame | 0.05 |

### Food and death

Food is **not stored as a counter** — it is modelled as a deadline. A player is created
with `death_time = now + 1260 / freq` seconds (10 food units × 126 time units). `Take food`
adds `126 / freq` seconds, `Set food` removes the same amount, and the reported food count
is `ceil(remaining_time / unit_duration)`. When `now >= death_time` the server sends `dead`
to the client, `pdi #id` to the GUI, and drops both the player and the connection.

### Teams, slots and eggs

- Each team starts with `-c` slots. Joining consumes one and spawns the player at a random position.
- `Fork` lays an `Egg` on the player's tile. A subsequent connection to that team consumes
  the **egg first** (emitting `ebo #id`) and spawns the player on the egg's tile.
- The number returned to a joining client (and by `Connect_nbr`) is
  `remaining team slots + pending eggs for that team`.
- Removing a player gives a slot back to their team.

### Elevation

`Incantation` is validated **twice**: once when the command is received, and again 300 time
units later, before the level is granted. Both checks require the stones to be present on
the tile *and* enough same-level players standing on it.

| Level | Players | linemate | deraumere | sibur | mendiane | phiras | thystame |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 → 2 | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| 2 → 3 | 2 | 1 | 1 | 1 | 0 | 0 | 0 |
| 3 → 4 | 2 | 2 | 0 | 1 | 0 | 2 | 0 |
| 4 → 5 | 4 | 1 | 1 | 2 | 0 | 1 | 0 |
| 5 → 6 | 4 | 1 | 2 | 1 | 3 | 0 | 0 |
| 6 → 7 | 6 | 1 | 2 | 3 | 0 | 1 | 0 |
| 7 → 8 | 6 | 2 | 2 | 2 | 2 | 2 | 1 |

On success the stones are consumed and **every** participating same-level player on the
tile is levelled up.

### Sound direction

`game::commands::compute_direction` maps a broadcast/eject source to the RFC's 1–8 tile
numbering (0 = same tile) using the **minimum image convention**: the `(dx, dy)` vector is
wrapped to its shortest representation on the torus, converted to a bearing with
`atan2(dx, -dy)`, snapped to one of 8 × 45° sectors, then rotated by the *receiver's*
orientation so that `1` is always the tile in front of them.

### Victory

Checked every tick: as soon as a team has **6 players at level 8**, the server broadcasts
`seg <team>` to graphic clients and exits its main loop.

## AI protocol

### Handshake

```
server → WELCOME
client → TEAM_NAME
server → <remaining slots + eggs>
server → <width> <height>
```

### Commands

| Command | Time (units) | Response |
| --- | --- | --- |
| `Forward` | 7 | `ok` |
| `Right` | 7 | `ok` |
| `Left` | 7 | `ok` |
| `Look` | 7 | `[ tile0, tile1, ... ]` |
| `Inventory` | 1 | `[ l N, d N, s N, m N, p N, t N, f N ]` |
| `Broadcast <text>` | 7 | `ok` |
| `Connect_nbr` | 0 | `<slots + eggs>` |
| `Fork` | 42 | `ok` |
| `Eject` | 7 | `ok` / `ko` |
| `Take <object>` | 7 | `ok` / `ko` |
| `Set <object>` | 7 | `ok` / `ko` |
| `Incantation` | 300 | `Elevation underway` then `Current level: N` / `ko` |

`<object>` is one of `food`, `linemate`, `deraumere`, `sibur`, `mendiane`, `phiras`, `thystame`.
Any unparsable line answers `ko`.

`Look` lists tiles from the player's own tile outwards, `level + 1` rows deep, each row
scanned left to right relative to the player's facing. A tile is a space-separated list of
`player` entries followed by its resources.

### Unsolicited messages

| Message | Sent when |
| --- | --- |
| `message K, text` | Another player broadcast; `K` is the incoming direction (0–8) |
| `eject K` | The player was ejected; `K` is the direction they were pushed from |
| `dead` | The player starved |

## GUI protocol

A client sends `GRAPHIC` instead of a team name. It immediately receives
`smg Welcome to Zappy Server!`, `msz`, `sgt`, a `bct` for **every** tile, one `tna` per team,
a `pnw` + `pin` pair per connected player, and an `enw` per pending egg.

### Requests

| Request | Reply |
| --- | --- |
| `msz` | `msz X Y` |
| `bct X Y` | `bct X Y f l d s m p t` (or `sbp` if out of bounds) |
| `mct` | one `bct` per tile |
| `tna` | one `tna N` per team |
| `ppo #n` | `ppo #n X Y O` (or `sbp`) |
| `plv #n` | `plv #n L` (or `sbp`) |
| `pin #n` | `pin #n X Y f l d s m p t` (or `sbp`) |
| `sgt` | `sgt T` |
| `sst T` | `sst T` — also rescales every pending deadline so the game speeds up/slows down live |
| *anything else* | `suc` |

Orientation `O` is `1 = North`, `2 = East`, `3 = South`, `4 = West`.

### Server-pushed events

| Event | Trigger |
| --- | --- |
| `pnw #n X Y O L T` | A player connected |
| `pin #n X Y f l d s m p t` | Inventory changed (join, `Take`, `Set`) |
| `ppo #n X Y O` | `Forward`, `Right`, `Left`, or being ejected |
| `plv #n L` | Elevation succeeded |
| `pgt #n R` / `pdr #n R` | Resource taken / dropped (`R` is `0..6`, food first) |
| `pbc #n text` | Broadcast |
| `pic X Y L #n #n ...` | Incantation started |
| `pie X Y R` | Incantation ended (`1` success, `0` failure) |
| `pfk #n` | `Fork` issued |
| `enw #e #n X Y` | Egg laid |
| `ebo #e` | Egg hatched into a player |
| `pex #n` | Player performed an eject |
| `pdi #n` | Player died or disconnected |
| `seg T` | End of game, team `T` wins |

## Bonus: TUI dashboard

Passing `-b` / `--bonus` runs the server on a background thread and takes over the terminal
with a Ratatui dashboard. Server logs and metrics travel over an `mpsc` channel as
`ServerEvent` values instead of being printed.

Two tabs:

- **Global Dashboard** — colour-coded live event stream, port/map/frequency/uptime metrics,
  per-team active player table, and a server-load gauge.
- **Live Minimap** — player positions drawn on a canvas of the map.

| Key | Action |
| --- | --- |
| `1` / `←` | Global Dashboard tab |
| `2` / `→` | Live Minimap tab |
| `c` | Clear the log pane |
| `q` | Quit |

## Tests

13 unit tests cover argument parsing, world creation and resource spawning, team slot
accounting, the food/time model, GUI command parsing, and GUI map/player/time responses.

```bash
cargo test
```

## Implementation notes

A few behaviours worth knowing when integrating an AI or a GUI against this server:

- **`Inventory` ordering.** The reply is `[ l N, d N, s N, m N, p N, t N, f N ]` — stones
  first with single-letter keys, food last — rather than the RFC's
  `[ food N, linemate N, ... ]`. Clients must parse this shape.
- **Queue overflow is silent.** An 11th pending command is dropped without a `ko`.
- **`Connect_nbr` is free.** It has a duration of 0 time units, so it still goes through the
  queue but completes on the next tick.
- **Incantation state is process-global.** `ACTIVE_INCANTATIONS` is a `static` map keyed by
  player id, shared across the whole process.
- **Slots are refunded on death.** A player removed for any reason returns a slot to their
  team, including one that originally hatched from an egg.
- **Eggs on GUI connect.** The initial `enw` burst reports the parent as `#0`, since the
  layer's id is not stored on the egg.
