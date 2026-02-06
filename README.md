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
- [🐳 Docker Deployment](#-docker-deployment)
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

### 🐳 Docker Ready
One-command deployment with Docker. Includes docker-compose for production use.

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

**From Source**
```bash
cargo install --git https://github.com/ricardodantas/tickit-sync
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
# Output: Generated token for 'my-laptop': tks_a1b2c3d4e5f6...
```

### 4. Start Server

```bash
# Start on default port 3030
tickit-sync serve

# Or specify port
tickit-sync serve --port 8080
```

### 5. Configure Tickit Client

Add to `~/.config/tickit/config.toml`:

```toml
[sync]
enabled = true
server = "http://your-server:3030"
token = "tks_a1b2c3d4e5f6..."
interval_secs = 300  # Sync every 5 minutes
```

<br>

## 🐳 Docker Deployment

### Quick Start

```bash
docker run -d \
  --name tickit-sync \
  -p 3030:3030 \
  -v tickit-data:/data \
  ricardodantas/tickit-sync
```

### Docker Compose (Recommended)

```yaml
# docker-compose.yml
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
docker compose up -d
```

### Generate Token in Docker

```bash
docker exec tickit-sync tickit-sync token --name "my-device"
```

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
port = 3030
host = "0.0.0.0"

# Database path
database = "/data/tickit-sync.sqlite"

# API tokens (managed via CLI, don't edit manually)
[[tokens]]
name = "my-laptop"
token = "tks_a1b2c3d4e5f6..."
created_at = "2026-02-06T22:00:00Z"

[[tokens]]
name = "my-desktop"
token = "tks_x9y8z7w6v5u4..."
created_at = "2026-02-06T23:00:00Z"
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
# Generate new token
tickit-sync token --name "device-name"

# List all tokens
tickit-sync token --list

# Revoke a token
tickit-sync token --revoke "device-name"
```

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
  "last_sync": "2026-02-06T22:00:00Z",  // null for first sync
  "changes": [
    {
      "type": "task",
      "data": {
        "id": "uuid",
        "title": "Buy groceries",
        "completed": false,
        "priority": "medium",
        "list_id": "uuid",
        "created_at": "2026-02-06T20:00:00Z",
        "updated_at": "2026-02-06T22:30:00Z"
      }
    },
    {
      "type": "deleted",
      "data": {
        "id": "uuid",
        "record_type": "task",
        "deleted_at": "2026-02-06T22:15:00Z"
      }
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
    completed INTEGER DEFAULT 0,
    priority TEXT DEFAULT 'medium',
    list_id TEXT,
    due_date TEXT,
    url TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    device_id TEXT NOT NULL
);

-- Lists
CREATE TABLE lists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    icon TEXT,
    is_inbox INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    device_id TEXT NOT NULL
);

-- Tags
CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    color TEXT,
    created_at TEXT NOT NULL,
    device_id TEXT NOT NULL
);

-- Sync state per device
CREATE TABLE sync_state (
    device_id TEXT PRIMARY KEY,
    last_sync TEXT NOT NULL
);

-- Tombstones for deleted records
CREATE TABLE tombstones (
    id TEXT PRIMARY KEY,
    record_type TEXT NOT NULL,
    deleted_at TEXT NOT NULL,
    device_id TEXT NOT NULL
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
2. **Keep tokens secret** - treat them like passwords
3. **Firewall** - only expose the server to trusted networks or use a VPN
4. **Backups** - regularly backup the SQLite database
5. **Updates** - keep tickit-sync updated for security patches

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
