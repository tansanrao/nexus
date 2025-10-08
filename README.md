# Linux KB

A comprehensive knowledge base for Linux kernel development, providing a powerful mailing list browser and search interface for navigating the Linux kernel community's communication history.

## Overview

Linux KB synchronizes and indexes Linux kernel mailing list archives from [lore.kernel.org](https://lore.kernel.org), enabling developers to:

- Browse and search mailing list threads with advanced filtering
- Track email conversations with intelligent threading
- Search by author, subject, date range, and content
- Analyze contributor activity and participation patterns
- Navigate patch series and RFC discussions

## Current Features

### Mailing List Browser
- **Multi-list support**: Track multiple mailing lists simultaneously (BPF, sched-ext, and more)
- **Intelligent threading**: JWZ threading algorithm with patch series detection
- **Advanced search**: Full-text search across subjects and email bodies
- **Author profiles**: View contributor statistics and participation history
- **Time zone support**: Display timestamps in your local time zone
- **Real-time sync**: Background synchronization with status tracking

## Tech Stack

### Backend
- **[Rust](https://www.rust-lang.org/)**: Systems programming language for performance and safety
- **[Rocket](https://rocket.rs/)**: Web framework for the REST API
- **[SQLx](https://github.com/launchbadge/sqlx)**: Async PostgreSQL driver with compile-time query validation
- **[gix](https://github.com/Byron/gitoxide)**: Pure Rust Git implementation for repository syncing
- **[mailparse](https://crates.io/crates/mailparse)**: RFC 2822 email parsing

### Frontend
- **[React](https://react.dev/)**: UI component library
- **[TypeScript](https://www.typescriptlang.org/)**: Type-safe JavaScript
- **[Vite](https://vitejs.dev/)**: Fast build tool and dev server
- **[TailwindCSS](https://tailwindcss.com/)**: Utility-first CSS framework
- **[React Router](https://reactrouter.com/)**: Client-side routing

### Database
- **[PostgreSQL](https://www.postgresql.org/)**: Primary data store with table partitioning
- **Partitioning strategy**: Tables partitioned by mailing list for optimal performance at scale
- **Full-text search**: Built-in PostgreSQL text search capabilities

### Data Pipeline
- **Git-based sync**: Clones public-inbox repositories from lore.kernel.org
- **Multi-repository support**: Handles numbered archives (/0, /1, /2) per mailing list
- **Email parsing**: Extracts headers, threading metadata, and patch series information
- **JWZ threading**: Industry-standard algorithm for conversation reconstruction

## Quick Start

### Option 1: Docker Deployment (Recommended)

The easiest way to get started is using Docker:

```bash
# Build and start all services
make build
make up

# Initialize database
make init

# View logs
make logs
```

**Services:**
- Frontend: http://localhost:80
- API Server: http://localhost:8000

See [DOCKER_DEPLOYMENT.md](./DOCKER_DEPLOYMENT.md) for detailed Docker deployment guide.

### Option 2: Manual Installation

#### Prerequisites
- Rust 1.70+ ([install](https://rustup.rs/))
- PostgreSQL 14+ ([install](https://www.postgresql.org/download/))
- Node.js 18+ ([install](https://nodejs.org/))
- Python 3.6+ with pip (for grokmirror)
- 20GB+ free disk space (for mailing list archives)

#### Grokmirror Setup

Linux KB uses grokmirror to efficiently mirror all lore.kernel.org repositories.

```bash
# Install grokmirror
pip install grokmirror

# Run initial mirror sync (this will take several hours on first run)
grok-pull -c grokmirror.conf

# Set up continuous syncing (choose one):
# Option 1: Systemd service (recommended - see GROKMIRROR_SETUP.md)
# Option 2: Cron job every 5 minutes
crontab -e
# Add: */5 * * * * cd /path/to/linux-kernel-kb && grok-pull -c grokmirror.conf
```

See [GROKMIRROR_SETUP.md](./GROKMIRROR_SETUP.md) for detailed setup instructions.

#### Database Setup

```bash
# Create database
createdb linux-kernel-kb

# Database schema is auto-created on first run via the reset endpoint
```

#### Backend Setup

```bash
cd api-server

# Configure database connection
cp Rocket.toml.example Rocket.toml
# Edit Rocket.toml to set your PostgreSQL credentials

# Build and run
cargo run --release
# API server runs on http://localhost:8000
```

#### Frontend Setup

```bash
cd frontend

# Install dependencies
npm install

# Start dev server
npm run dev
# Frontend runs on http://localhost:5173
```

#### Initial Data Sync

1. **Ensure grokmirror is running** and has completed at least one sync
2. Navigate to http://localhost:5173/settings
3. Go to the "Database" panel and click "Reset Database" to initialize schema
4. Click "Seed Mailing Lists" to populate all ~341 lore.kernel.org lists (default: all disabled)
5. Go to the "Mailing Lists" panel and enable the lists you want to parse
6. Click "Sync" to start importing emails from the enabled lists
7. Monitor progress in the "Sync Status" panel

**Note**: Grokmirror mirrors ALL lists continuously. The "enabled" toggle only controls which lists the API server will parse and import.

## Configuration

### Environment Variables (Backend)

```bash
# Database connection (or configure in Rocket.toml)
DATABASE_URL=postgres://user:password@localhost/linux-kernel-kb

# Mirror storage location
MIRROR_BASE_PATH=./mirrors

# Logging level
RUST_LOG=info
```

### Mailing Lists

Linux KB supports all ~341 mailing lists archived by lore.kernel.org, including:
- **lkml**: Linux Kernel Mailing List (18 epochs/shards)
- **netdev**: Network device development (2 epochs)
- **bpf**: Berkeley Packet Filter development
- **dri-devel**: DRI development
- **linux-fsdevel**: Filesystem development
- And 336 more...

All lists are mirrored by grokmirror. Use the Settings UI to enable parsing for specific lists.

## Architecture

### Database Schema

Tables are partitioned by mailing list for optimal performance:

- `mailing_lists`: Mailing list metadata and configuration
- `mailing_list_repositories`: Git repository URLs (supports multiple repos per list)
- `authors_{slug}`: Author information (partitioned per mailing list)
- `emails_{slug}`: Email messages (partitioned per mailing list)
- `threads_{slug}`: Thread metadata (partitioned per mailing list)
- `email_recipients_{slug}`: To/Cc relationships (partitioned per mailing list)
- `email_references_{slug}`: Email reference chains (partitioned per mailing list)
- `thread_memberships_{slug}`: Thread membership and depth (partitioned per mailing list)

### Sync Pipeline

Linux KB uses a two-phase sync architecture:

**Phase 1: Mirror Sync (Grokmirror - External)**
1. Grokmirror runs continuously (cron/systemd)
2. Mirrors ALL ~341 lore.kernel.org repositories to local disk
3. Uses rsync-like delta transfers for efficiency
4. Independent of API server lifecycle

**Phase 2: Email Import (API Server - Internal)**
1. **Commit Discovery**: Traverse git history to find new email commits (incremental)
2. **Email Parsing**: Extract headers, body, and metadata from each commit
3. **Import**: Batch insert authors, emails, recipients, and references
4. **Threading**: Apply JWZ algorithm to build conversation threads
5. **Indexing**: PostgreSQL full-text search on subjects and bodies

Only enabled mailing lists are imported. Subsequent syncs process only new commits via `last_indexed_commit` tracking.

## API Documentation

The REST API is available at `http://localhost:8000/api`:

### Mailing Lists
- `GET /api/mailing-lists` - List all mailing lists
- `GET /api/mailing-lists/:slug` - Get mailing list details

### Per-Mailing-List Endpoints
- `GET /api/:slug/threads` - List threads (paginated, filterable)
- `GET /api/:slug/threads/:id` - Get thread with all emails
- `GET /api/:slug/emails/:id` - Get single email
- `GET /api/:slug/authors` - Search authors
- `GET /api/:slug/authors/:id` - Get author profile
- `GET /api/:slug/stats` - Get mailing list statistics

### Admin Endpoints
- `POST /api/admin/sync/:slug/start` - Start sync for mailing list
- `GET /api/admin/sync/:slug/status` - Get sync status
- `POST /api/admin/database/reset` - Reset entire database

## Development

### Running Tests

```bash
cd api-server
cargo test
```

### Code Style

```bash
# Backend
cargo fmt
cargo clippy

# Frontend
npm run lint
```

## Roadmap

- [ ] Add support for all lore.kernel.org mailing lists
- [ ] Advanced search with boolean operators
- [ ] Email thread visualization
- [ ] Patch series tracking and review workflow
- [ ] Export threads as mbox format
- [ ] Author network analysis
- [ ] Bookmark and annotation system

## Contributing

This project is under active development. Contributions welcome!

## License

TBD

## Acknowledgments

- [lore.kernel.org](https://lore.kernel.org) for maintaining comprehensive kernel mailing list archives
- [public-inbox](https://public-inbox.org/) for the Git-based archive format
- JWZ threading algorithm by Jamie Zawinski
