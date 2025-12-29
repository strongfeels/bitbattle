# BitBattle

A real-time competitive coding platform where players battle head-to-head solving programming challenges.

## Overview

BitBattle allows players to join rooms and compete in live coding battles. Race against others to solve problems first, track your ELO rating across difficulty levels, and climb the global leaderboard.

### Features

- **Real-time Battles** - See opponents' progress live via WebSocket
- **Room-based Matches** - Create or join rooms with unique codes (e.g., `SWIFT-CODER-1234`)
- **Google OAuth** - Sign in to save stats and compete in ranked matches
- **ELO Rating System** - Separate ratings for Easy, Medium, and Hard problems
- **Casual & Ranked Modes** - Practice without pressure or compete for rating
- **Global Leaderboard** - Rankings by difficulty with win rates and peak ratings
- **Player Profiles** - View stats, game history, and achievements
- **Multi-language Support** - JavaScript and Python

## Tech Stack

### Frontend
- React 19 with TypeScript
- Vite (build tool)
- Tailwind CSS (minimalist dark theme)
- CodeMirror 6 (code editor)
- React Router for navigation
- WebSocket for real-time sync

### Backend
- Rust with Axum web framework
- PostgreSQL with SQLx
- Google OAuth 2.0 + JWT authentication
- Tokio async runtime
- Docker-based sandboxed code execution

## Getting Started

### Prerequisites

- Node.js (v18+)
- Rust toolchain
- PostgreSQL
- Docker (for code execution)

### Environment Setup

Create `.env` in `bitbattle-backend/`:

```env
DATABASE_URL=postgres://user:password@localhost:5432/bitbattle
GOOGLE_CLIENT_ID=your-google-client-id.apps.googleusercontent.com
GOOGLE_CLIENT_SECRET=your-google-client-secret
GOOGLE_REDIRECT_URI=http://localhost:4000/auth/callback
JWT_SECRET=your-secret-key-at-least-32-characters
FRONTEND_URL=http://localhost:5173
```

### Database Setup

```bash
# Create database
psql -U postgres -c "CREATE DATABASE bitbattle;"

# Migrations run automatically on server start
```

### Quick Start with Docker Compose

```bash
# Start PostgreSQL and build sandbox
docker-compose up -d

# Build sandbox image
docker build -t bitbattle-sandbox:latest ./bitbattle-backend/sandbox
```

### Manual Installation

**Backend:**

```bash
cd bitbattle-backend

# Build sandbox Docker image (required for code execution)
docker build -t bitbattle-sandbox:latest ./sandbox

# Run the server
cargo run
```

Server starts on `http://localhost:4000`

**Frontend:**

```bash
cd bitbattle-frontend
npm install
npm run dev
```

Dev server starts on `http://localhost:5173`

## Project Structure

```
bitbattle/
├── bitbattle-backend/
│   ├── src/
│   │   ├── main.rs              # Server setup, routes, WebSocket
│   │   ├── config.rs            # Environment configuration
│   │   ├── db.rs                # Database connection pool
│   │   ├── problems.rs          # Problem definitions
│   │   ├── executor.rs          # Sandboxed code execution
│   │   ├── auth/
│   │   │   ├── jwt.rs           # JWT token handling
│   │   │   └── middleware.rs    # Auth middleware
│   │   ├── handlers/
│   │   │   ├── auth.rs          # OAuth endpoints
│   │   │   ├── leaderboard.rs   # Leaderboard API
│   │   │   └── user.rs          # Profile & history
│   │   └── models/
│   │       ├── user.rs          # User & stats models
│   │       └── game_result.rs   # Game history model
│   ├── migrations/              # SQL migrations
│   └── sandbox/                 # Docker sandbox config
│
├── bitbattle-frontend/
│   ├── src/
│   │   ├── App.tsx              # Router & layout
│   │   ├── components/
│   │   │   ├── RoomLobby.tsx    # Home & room creation
│   │   │   ├── CollaborativeEditor.tsx
│   │   │   ├── Leaderboard.tsx  # Global rankings
│   │   │   ├── Profile.tsx      # User stats & history
│   │   │   ├── NavBar.tsx       # Navigation
│   │   │   └── Logo.tsx         # Butterfly BB logo
│   │   ├── contexts/
│   │   │   └── AuthContext.tsx  # Auth state management
│   │   └── utils/
│   │       └── api.ts           # API helpers with auth
│   └── package.json
│
└── docker-compose.yml
```

## How It Works

1. **Sign in or Play as Guest** - Google login saves your stats, or play as `guest_XXXX`
2. **Create or Join a Room** - Choose difficulty, player count (2-4), and game mode
3. **Battle** - Race to solve the problem first
4. **Win or Learn** - First to pass all tests wins; ELO updates for ranked games

## API Endpoints

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/auth/google` | GET | No | Start Google OAuth flow |
| `/auth/callback` | GET | No | OAuth callback, returns JWT |
| `/auth/me` | GET | Yes | Get current user |
| `/leaderboard` | GET | No | Get ranked players |
| `/leaderboard/:difficulty` | GET | No | Leaderboard by difficulty |
| `/users/:id/profile` | GET | No | Get user profile & stats |
| `/users/:id/history` | GET | No | Get game history |

## ELO Rating System

- Starting rating: 1200
- Separate ratings for Easy, Medium, and Hard
- K-factor: 32 (standard chess K-factor)
- Only ranked games affect rating
- Peak rating tracked per difficulty

## Security

Code execution is sandboxed with:

- **Network isolation** - No network access
- **Memory limit** - 128MB
- **CPU limit** - 0.5 cores
- **Process limit** - 50 max (prevents fork bombs)
- **Read-only filesystem** - Only `/tmp` writable
- **Non-root user** - Runs as `runner`
- **5-second timeout**

## Scripts

### Frontend

| Command | Description |
|---------|-------------|
| `npm run dev` | Start dev server |
| `npm run build` | Production build |
| `npm run preview` | Preview build |

### Backend

| Command | Description |
|---------|-------------|
| `cargo run` | Start server |
| `cargo build --release` | Production build |

## License

MIT
