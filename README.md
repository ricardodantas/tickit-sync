<h1 align="center">
  ☁️ Tickit Sync Server
</h1>

<p align="center">
  <strong>Self-hosted sync server for Tickit task manager</strong>
</p>

<p align="center">
  <i>Keep your tasks synced across all your devices — on your own infrastructure.</i>
</p>

<p align="center">
  <a href="https://github.com/ricardodantas/tickit-sync/releases">
    <img src="https://img.shields.io/github/v/release/ricardodantas/tickit-sync?style=flat&labelColor=1e1e2e&color=cba6f7&logo=github&logoColor=white" alt="Release">
  </a>
  <a href="https://crates.io/crates/tickit-sync">
    <img src="https://img.shields.io/crates/v/tickit-sync?style=flat&labelColor=1e1e2e&color=fab387&logo=rust&logoColor=white" alt="Crates.io">
  </a>
  <a href="https://hub.docker.com/r/ricardodantas/tickit-sync">
    <img src="https://img.shields.io/docker/v/ricardodantas/tickit-sync?style=flat&labelColor=1e1e2e&color=89b4fa&logo=docker&logoColor=white&label=docker" alt="Docker">
  </a>
  <a href="https://github.com/ricardodantas/tickit-sync/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-89b4fa?style=flat&labelColor=1e1e2e" alt="License">
  </a>
  <a href="https://rust-lang.org">
    <img src="https://img.shields.io/badge/rust-1.93+-f9e2af?style=flat&labelColor=1e1e2e&logo=rust&logoColor=white" alt="Rust Version">
  </a>
</p>

<br>

## 📖 Table of Contents

- [✨ Features](#-features)
- [🚀 Quick Start](#-quick-start)
- [🐳 Docker / Podman Deployment](#-docker--podman-deployment)
- [⚙️ Configuration](#️-configuration)
- [🔐 Authentication](#-authentication)
- [📡 API Reference](#-api-reference)
- [🏗️ Architecture](#️-architecture)
- [🔧 Building from Source](#-building-from-source)
- [🤝 Contributing](#-contributing)
- [📄 License](#-license)

<br>

## ✨ Features

<table>
<tr>
<td width="50%">

### 🔒 Self-Hosted
Your data stays on your infrastructure. No third-party services, no data mining, complete privacy.

### ⚡ Lightweight
Single binary, ~5MB. SQLite storage. Runs on anything from a Raspberry Pi to a cloud VM.

### 🔑 Token Auth
Simple API token authentication. Generate tokens per-device for easy management.

</td>
<td width="50%">

### 🐳 Docker/Podman Ready
One-command deployment with Docker or Podman. Includes docker-compose for production use.

### 🔄 Conflict Resolution
Last-write-wins with timestamp-based conflict detection. Your most recent changes always win.

### 📊 Multi-Device
Sync unlimited devices. Each device gets its own token for tracking and security.

</td>
</tr>
</table>

<br>

### How It Works

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Laptop    │     │   Desktop   │     │   Phone     │
│   Tickit    │     │   Tickit    │     │   (future)  │
└──────┬──────┘     └──────┬──────┘     └──────┬──────┘
       │                   │                   │
       │    HTTPS + Token  │                   │
       └───────────┬───────┴───────────────────┘
                   │
                   ▼
          ┌───────────────┐
          │  tickit-sync  │
          │    Server     │
          │   (SQLite)    │
          └───────────────┘
```

<br>

## 🚀 Quick Start

### 1. Install

**From Binary (Recommended)**
```bash
# Download latest release
curl -LO https://github.com/ricardodantas/tickit-sync/releases/latest/download/tickit-sync-linux-x86_64.tar.gz
tar xzf tickit-sync-linux-x86_64.tar.gz
sudo mv tickit-sync /usr/local/bin/
```

**From crates.io**
```bash
cargo install tickit-sync
```

### 2. Initialize

```bash
# Create config file
tickit-sync init

# This creates ~/.config/tickit-sync/config.toml
```

### 3. Generate Token

```bash
# Create a token for your first device
tickit-sync token --name "my-laptop"
# Token is auto-saved to config (hashed) and displays setup instructions
# for both mobile app and desktop CLI
```

> ⚠️ **Important:** If the server is already running, you must restart it after generating a new token:
> ```bash
> # Docker
> docker restart tickit-sync
> 
> # Podman
> podman restart tickit-sync
> ```

### 4. Start Server

```bash
# Start on default port 3030
tickit-sync serve

# Or specify port
tickit-sync serve --port 8080
```

### 5. Configure Tickit Clients

The token generation command shows setup instructions for all clients:

**Mobile App (tickit-mobile):**
- Open Settings
- Set Sync Server URL
- Paste the generated token
- Enable Sync

**Desktop CLI (tickit):**
- Press `s` to open Settings
- Navigate to Sync Server, press Enter, type URL
- Navigate to Sync Token, press Enter, paste token
- Enable Sync

Or manually add to `~/.config/tickit/config.toml`:

```toml
[sync]
enabled = true
server = "http://your-server:3030"
token = "tks_a1b2c3d4e5f6..."
interval_secs = 300  # Sync every 5 minutes
```

<br>

## 🐳 Docker / Podman Deployment

> **Note:** All commands below work with both Docker and Podman. Simply replace `docker` with `podman` if you prefer Podman.

### Quick Start

```bash
# Docker
docker run -d \
  --name tickit-sync \
  -p 3030:3030 \
  -v tickit-data:/data \
  ricardodantas/tickit-sync

# Podman
podman run -d \
  --name tickit-sync \
  -p 3030:3030 \
  -v tickit-data:/data \
  docker.io/ricardodantas/tickit-sync
```

### Docker/Podman Compose (Recommended)

```yaml
# docker-compose.yml / podman-compose.yml
services:
  tickit-sync:
    image: ricardodantas/tickit-sync:latest
    container_name: tickit-sync
    restart: unless-stopped
    ports:
      - "3030:3030"
    volumes:
      - ./data:/data
    environment:
      - TICKIT_SYNC_PORT=3030
```

```bash
# Docker
docker compose up -d

# Podman
podman-compose up -d
```

### Generate Token in Container

```bash
# Docker
docker exec tickit-sync tickit-sync token --name "my-device"

# Podman
podman exec tickit-sync tickit-sync token --name "my-device"

# The token is automatically saved to the server config.
# The output shows setup instructions for mobile app and desktop CLI.
```

> ⚠️ **Important:** After generating a token, restart the container for it to take effect:
> ```bash
> docker restart tickit-sync   # or: podman restart tickit-sync
> ```

### With Reverse Proxy (Caddy)

```
# Caddyfile
sync.yourdomain.com {
    reverse_proxy tickit-sync:3030
}
```

### With Traefik

```yaml
services:
  tickit-sync:
    image: ricardodantas/tickit-sync:latest
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.tickit-sync.rule=Host(`sync.yourdomain.com`)"
      - "traefik.http.routers.tickit-sync.tls.certresolver=letsencrypt"
    volumes:
      - ./data:/data
```

<br>

## ⚙️ Configuration

Configuration file: `~/.config/tickit-sync/config.toml` (or `/data/config.toml` in Docker)

```toml
# Server settings
[server]
port = 3030
bind = "0.0.0.0"

# Database settings
[database]
path = "/data/tickit-sync.sqlite"

# API tokens (managed via CLI, hashed with argon2)
[[tokens]]
name = "my-laptop"
token_hash = "$argon2id$v=19$m=19456,t=2,p=1$..."

[[tokens]]
name = "my-desktop"
token_hash = "$argon2id$v=19$m=19456,t=2,p=1$..."
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `TICKIT_SYNC_PORT` | `3030` | Server port |
| `TICKIT_SYNC_HOST` | `0.0.0.0` | Bind address |
| `TICKIT_SYNC_DB` | `./tickit-sync.sqlite` | Database path |
| `TICKIT_SYNC_CONFIG` | `~/.config/tickit-sync/config.toml` | Config file path |

<br>

## 🔐 Authentication

All API endpoints (except `/health`) require a Bearer token.

### Token Management

```bash
# Generate new token (automatically saved to config, hashed with argon2)
tickit-sync token --name "device-name"

# List all tokens
tickit-sync token --list

# Revoke a token
tickit-sync token --revoke "device-name"
```

> ⚠️ **Important:** Tokens are hashed with Argon2 before storage. The plaintext token is only shown once when generated. Save it immediately!

### Using Tokens

Include the token in the `Authorization` header:

```bash
curl -H "Authorization: Bearer tks_your_token_here" \
  https://sync.example.com/api/v1/sync
```

### Token Format

Tokens are prefixed with `tks_` followed by 32 random alphanumeric characters:
```
tks_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
```

Tokens are stored hashed in the config file using Argon2id:
```toml
[[tokens]]
name = "my-laptop"
token_hash = "$argon2id$v=19$m=19456,t=2,p=1$..."
```

<br>

## 📡 API Reference

### Health Check

```http
GET /health
```

**Response:**
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

### Sync

```http
POST /api/v1/sync
Authorization: Bearer <token>
Content-Type: application/json
```

**Request:**
```json
{
  "device_id": "uuid-of-device",
  "last_sync": "2026-02-06T22:00:00Z",
  "changes": [
    {
      "type": "list",
      "id": "uuid",
      "name": "Work",
      "icon": "💼",
      "is_inbox": false,
      "sort_order": 0,
      "created_at": "2026-02-06T20:00:00Z",
      "updated_at": "2026-02-06T22:30:00Z"
    },
    {
      "type": "task",
      "id": "uuid",
      "title": "Buy groceries",
      "completed": false,
      "priority": "medium",
      "list_id": "uuid",
      "tag_ids": ["tag-uuid-1", "tag-uuid-2"],
      "created_at": "2026-02-06T20:00:00Z",
      "updated_at": "2026-02-06T22:30:00Z"
    },
    {
      "type": "deleted",
      "id": "uuid",
      "record_type": "task",
      "deleted_at": "2026-02-06T22:15:00Z"
    }
  ]
}
```

**Response:**
```json
{
  "server_time": "2026-02-06T22:35:00Z",
  "changes": [
    // Changes from other devices since last_sync
  ],
  "conflicts": []  // Reserved for future conflict reporting
}
```

### Record Types

| Type | Description |
|------|-------------|
| `task` | Task record with title, description, priority, etc. |
| `list` | List/folder for organizing tasks |
| `tag` | Tag for categorizing tasks |
| `task_tag` | Association between task and tag |
| `deleted` | Tombstone for deleted records |

<br>

## 🏗️ Architecture

```
.
├── src/
│   ├── main.rs        # CLI entry point (clap)
│   ├── api.rs         # Axum HTTP handlers
│   ├── config.rs      # TOML config loading
│   ├── db.rs          # SQLite operations
│   └── models.rs      # Shared data types
├── Dockerfile         # Multi-stage build
├── docker-compose.yml # Production deployment
└── Cargo.toml
```

### Tech Stack

| Component | Technology |
|-----------|------------|
| Runtime | Rust (Edition 2024) |
| HTTP Framework | [Axum](https://github.com/tokio-rs/axum) |
| Database | SQLite via [rusqlite](https://github.com/rusqlite/rusqlite) |
| Async Runtime | [Tokio](https://tokio.rs/) |
| CLI Parser | [Clap](https://github.com/clap-rs/clap) |
| Serialization | [Serde](https://serde.rs/) + JSON |
| Config | TOML |

### Database Schema

```sql
-- Tasks synced from all devices
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    url TEXT,
    priority TEXT DEFAULT 'medium',
    completed INTEGER DEFAULT 0,
    list_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    completed_at TEXT,
    due_date TEXT,
    FOREIGN KEY (list_id) REFERENCES lists(id)
);

-- Lists/folders for organizing tasks
CREATE TABLE lists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT DEFAULT '📁',
    color TEXT,
    is_inbox INTEGER DEFAULT 0,
    sort_order INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Tags for categorizing tasks
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT
);

-- Task-Tag junction table
CREATE TABLE task_tags (
    task_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (task_id, tag_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Tombstones for deleted records
CREATE TABLE tombstones (
    id TEXT PRIMARY KEY,
    record_type TEXT NOT NULL,
    deleted_at TEXT NOT NULL
);

-- Device sync state tracking
CREATE TABLE device_sync (
    device_id TEXT PRIMARY KEY,
    last_sync TEXT NOT NULL
);
```

<br>

## 🔧 Building from Source

### Requirements

- **Rust 1.93+** (uses Edition 2024 features)
- **SQLite** development libraries

### Build

```bash
# Clone
git clone https://github.com/ricardodantas/tickit-sync
cd tickit-sync

# Build release binary
cargo build --release

# Binary at: target/release/tickit-sync
```

### Build Docker Image

```bash
docker build -t tickit-sync .
```

### Cross-Compile

```bash
# For Linux (musl - static binary)
cargo build --release --target x86_64-unknown-linux-musl

# For macOS
cargo build --release --target x86_64-apple-darwin

# For Windows
cargo build --release --target x86_64-pc-windows-msvc
```

<br>

## 🔒 Security Considerations

1. **Always use HTTPS** in production (via reverse proxy)
2. **Tokens are hashed** - stored using Argon2id, never in plaintext
3. **Keep tokens secret** - treat them like passwords, only shown once at generation
4. **Firewall** - only expose the server to trusted networks or use a VPN
5. **Backups** - regularly backup the SQLite database
6. **Updates** - keep tickit-sync updated for security patches

<br>

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

<br>

## 📄 License

MIT License - see [LICENSE](LICENSE) for details.

<br>

---

<p align="center">
  <sub>Built with ❤️ for <a href="https://github.com/ricardodantas/tickit">Tickit</a></sub>
</p>
