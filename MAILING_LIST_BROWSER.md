# Linux Kernel Mailing List Browser

A full-stack application for browsing the BPF mailing list with a Rust/Rocket backend and React frontend.

## Architecture

- **Backend**: Rust + Rocket web framework + SQLx (PostgreSQL)
- **Frontend**: React + TypeScript + Vite + Tailwind CSS
- **Database**: PostgreSQL (populated via the `poc/poc.ipynb` notebook)

## Project Structure

```
.
├── api-server/          # Rust/Rocket API backend
│   ├── src/
│   │   ├── main.rs      # Server entry point
│   │   ├── db.rs        # Database connection
│   │   ├── models.rs    # Data models
│   │   └── routes/      # API endpoints
│   ├── Cargo.toml       # Rust dependencies
│   └── Rocket.toml      # Rocket configuration
├── frontend/            # React frontend
│   ├── src/
│   │   ├── App.tsx      # Main app with routing
│   │   ├── api/         # API client
│   │   ├── components/  # React components
│   │   └── types/       # TypeScript types
│   └── package.json     # Node dependencies
└── poc/
    └── poc.ipynb        # Database setup & parsing script
```

## Prerequisites

- Rust (latest stable)
- Node.js 18+ and npm
- PostgreSQL 14+
- Python 3.8+ (for running the notebook)

## Setup

### 1. Database Setup

First, ensure PostgreSQL is running and create the database using the Jupyter notebook:

```bash
cd poc
jupyter notebook poc.ipynb
# Run all cells to create and populate the database
```

This will:
- Mirror the BPF mailing list from public-inbox
- Create the PostgreSQL schema
- Parse all emails and populate the database

### 2. Backend Setup

```bash
cd api-server

# Build the backend
cargo build --release

# Run the server
cargo run --release
```

The API server will start on `http://localhost:8000`

### API Endpoints

- `GET /api/threads?page=1&limit=50` - List threads (paginated)
- `GET /api/threads/:id` - Get thread with all emails
- `GET /api/emails/:id` - Get single email
- `GET /api/authors?search=query&page=1` - Search authors
- `GET /api/authors/:id` - Get author profile
- `GET /api/authors/:id/emails` - Get author's emails
- `GET /api/stats` - Database statistics

### 3. Frontend Setup

```bash
cd frontend

# Install dependencies
npm install

# Run development server
npm run dev
```

The frontend will start on `http://localhost:5173`

## Features

### Thread Browser
- Browse mailing list threads chronologically
- View thread metadata (message count, dates)
- Pagination support

### Thread View
- Hierarchical email display with visual indentation
- Shows reply depth
- Full email headers and body
- Links to author profiles

### Author Explorer
- Search authors by name or email
- View author statistics (email count, thread participation)
- Browse author's email history
- Activity timeline

### Database Schema

**Tables:**
- `authors` - Email addresses and names
- `emails` - Email messages with full content
- `threads` - Thread metadata
- `thread_memberships` - Email-to-thread relationships
- `email_recipients` - To/CC recipients
- `email_references` - Email reference chains

## Configuration

### Backend Configuration

Edit `api-server/Rocket.toml`:

```toml
[default]
address = "127.0.0.1"
port = 8000

[default.databases.bpf_db]
url = "postgres://postgres:example@localhost:5432/linux-kernel-kb"
```

### Frontend Configuration

Edit `frontend/src/api/client.ts` to change the API base URL if needed:

```typescript
const API_BASE_URL = 'http://localhost:8000/api';
```

## Development

### Backend

```bash
# Run in development mode with auto-reload
cd api-server
cargo watch -x run

# Run tests
cargo test

# Check for errors
cargo check
```

### Frontend

```bash
cd frontend

# Development server with hot reload
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## Technology Stack

### Backend
- **Rocket 0.5** - Web framework
- **SQLx 0.7** - Async PostgreSQL driver
- **Serde** - Serialization/deserialization
- **Chrono** - Date/time handling
- **rocket_cors** - CORS support

### Frontend
- **React 18** - UI library
- **TypeScript** - Type safety
- **Vite** - Build tool
- **React Router** - Client-side routing
- **Tailwind CSS** - Styling

## License

MIT
