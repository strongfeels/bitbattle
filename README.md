# BitBattle

A real-time collaborative coding challenge platform where users compete in live coding battles together.

## Overview

BitBattle allows multiple users to join virtual "rooms" and work on coding problems simultaneously. Users can see real-time code changes from other participants, submit solutions, and receive instant feedback on test results.

### Features

- **Real-time Collaboration** - See other participants' code changes instantly via WebSocket
- **Room-based Battles** - Join rooms using unique codes (e.g., `SWIFT-CODER-1234`)
- **Coding Challenges** - Problems of varying difficulty (Easy, Medium, Hard)
- **Live Code Execution** - Submit solutions and get instant test results
- **Multi-user Presence** - Track who's in the room and their progress

## Tech Stack

### Frontend
- React 19 with TypeScript
- Vite (build tool)
- Tailwind CSS
- CodeMirror 6 (code editor)
- WebSocket for real-time communication

### Backend
- Rust with Axum web framework
- Tokio async runtime
- WebSocket support
- Node.js for JavaScript code execution

## Getting Started

### Prerequisites

- Node.js (v18+)
- Rust toolchain
- npm or yarn

### Installation

**Backend:**

```bash
cd bitbattle-backend
cargo build
cargo run
```

The server will start on `http://localhost:4000`

**Frontend:**

```bash
cd bitbattle-frontend
npm install
npm run dev
```

The dev server will start on `http://localhost:5173`

## Project Structure

```
bitbattle/
├── bitbattle-backend/           # Rust backend server
│   ├── src/
│   │   ├── main.rs             # Server setup, routes, WebSocket handler
│   │   ├── problems.rs         # Problem definitions
│   │   └── executor.rs         # Code execution engine
│   └── Cargo.toml
│
└── bitbattle-frontend/          # React frontend
    ├── src/
    │   ├── App.tsx             # Main app component
    │   ├── components/
    │   │   ├── CollaborativeEditor.tsx  # Main editor interface
    │   │   ├── CodeMirrorEditor.tsx     # Code editor wrapper
    │   │   ├── ProblemPanel.tsx         # Problem display
    │   │   └── RoomLobby.tsx            # Room creation/join UI
    │   ├── hooks/
    │   │   └── useWebSocket.ts          # WebSocket connection hook
    │   └── utils/
    │       └── roomUtils.ts             # Room code utilities
    └── package.json
```

## How It Works

1. **Create or Join a Room** - Generate a new room code or enter an existing one
2. **Get Assigned a Problem** - Each room is assigned a random coding challenge
3. **Write Your Solution** - Use the built-in code editor with syntax highlighting
4. **Submit & Test** - Your code runs against hidden test cases
5. **See Results** - Test results are broadcast to all room participants

## Available Problems

- **Two Sum** (Easy) - Find two numbers that sum to a target
- **Reverse String** (Easy) - Reverse a string array in-place
- **Valid Parentheses** (Easy) - Validate bracket matching

## Scripts

### Frontend

| Command | Description |
|---------|-------------|
| `npm run dev` | Start development server with HMR |
| `npm run build` | Build for production |
| `npm run lint` | Run ESLint |
| `npm run preview` | Preview production build |

### Backend

| Command | Description |
|---------|-------------|
| `cargo run` | Start the server |
| `cargo build --release` | Build for production |
| `cargo test` | Run tests |

## Configuration

- Backend server runs on port `4000` by default
- Code execution has a 5-second timeout
- Currently supports JavaScript (Python planned)

## License

MIT
