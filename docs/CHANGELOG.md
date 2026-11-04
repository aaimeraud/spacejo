# Journal des versions

Ce fichier est GÉNÉRÉ par git-cliff à chaque release (scripts/release.sh) : ne pas l'éditer à la main.

## 1.4.1 - 2026-11-04

### Corrections

- **web** : Leave room under the iPhone status bar in the installed app (a008619)

## 1.4.0 - 2026-11-04

### Nouveautés

- **web** : Show the latest release and commit from GitHub on the home screen (444beb3)
- **core** : Add a reduced deck option proportional to the number of players (988ae32)
- **web** : Offer the reduced deck and animate the discard reshuffle (715ac77)

## 1.3.0 - 2026-11-04

### Nouveautés

- **api** : Add an in-game chat with messages pushed to every player (136fe23)
- **web** : Add a chat panel to the game screen with an unread counter (f503077)

## 1.2.0 - 2026-11-04

### Nouveautés

- **api** : Let the host switch a lobby between private and public (8b2b078)
- **web** : Switch visibility and show the invite link as a QR code (aff864b)
- **api** : Let players leave a game and let the host kick from the lobby (62539a6)
- **api** : Rate-limit game creation, joining and the public games list per IP (4244895)
- **web** : Show departures to everyone, kick from the lobby, retry flips (9816ea7)

## 1.1.0 - 2026-11-04

### Nouveautés

- **api** : List public games and let hosts create public or private games (d08413b)
- **web** : Share an invite link and join public games from the home screen (6548719)

## 1.0.0 - 2026-11-04

### Nouveautés

- **core** : Implement game rules, scoring and filtered view (3bd52f1)
- **api** : Add HTTP API, tokens, PostgreSQL store and migrations (62a0c63)
- **web** : Add React interface for lobby, game and final ranking (e417bd0)
- **web** : Show the game as a 3D table with flipping cards (d444159)
- **web** : Tilt the table more, zoom the active seat, animate cleared columns (0ae9161)
- **web** : Turn opponents' grids toward their owners, soften the tilt (3178929)
- **web** : Choose what to do with a drawn card with styled tiles (38419ba)
- **web** : Redesign game settings with presets and rule cards (eb8432b)
- **core** : Allow up to 10 players and eliminate absent players (df638d6)
- **api** : Track player presence and skip absent players' turns (ed91edd)
- **web** : Resume games and show absent or eliminated players (5cab649)
- **web** : Fixed no-scroll screens, step-by-step setup and Lucide icons (37b401d)
- **web** : Lay opponent grids flat on the table with upright labels (33af42c)
- **api** : Let the host skip an absent player's turn (ed0d98c)
- **web** : Add a skip button to the absent-player banner (8542345)
- **web** : Restyle the interface with a neutral night theme (aaa41ab)
- **web** : Drop the top bar and the floating home panel (c7576bf)
- **web** : Add a light theme with a theme toggle (1e31d91)
- **web** : Show one opponent at a time on phones (f50900e)
- **web** : Make the app installable with icons and link previews (883d864)
- **db** : Support the Supabase transaction pooler (ebda59d)
- **api** : Replace the Vercel functions with a single axum server (a3884b4)
- **api** : Push game updates to players with server-sent events (e7e86cd)

### Corrections

- **api** : Pin the Rust toolchain so Vercel can build sqlx 0.9 (8566bc8)

### Performances

- **web** : Poll less often when nothing is happening (25ecbb9)
- **api** : Lighten the release profile to speed up Vercel builds (33f37be)
- **db** : Cut database round trips per request (7550e4c)

### Documentation

- Add game rules, API contract and v1 plan (55b4b99)
- Add README, release decision and command list (60fc1c4)
- Fix body-size constant location and close the v1 plan (9237307)

### Tests

- **core** : Specify game rules with failing tests (703246e)

### Intégration continue

- Add CI and tag-triggered release workflows (e80475d)

### Style

- **web** : Reformat game logic helpers (94a66bb)
- **web** : Center the turn hint text (894df70)

### Maintenance

- Add commit checks and documentation skeleton (0e8b9fe)
- Add release scripts, git-cliff config and deployment config (b38f5be)
- Ignore local preview configuration (33d1718)
- Ignore design-tool working files (2a707e9)
- **ci** : Run the functions in Dublin, next to the database (12fe875)
- **ci** : Add the Docker image and the Render blueprint (09dfd3a)
- Move to Rust 1.99.1, Node 26 LTS and lucide-react 1.66 (d70b9ec)

<!-- généré par git-cliff -->
