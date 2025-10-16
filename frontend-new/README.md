# Nexus Frontend

A modern React-based mailing list browser with a Hacker News-style interface for viewing email threads and patches.

## Features

- 🎨 Modern UI with shadcn/ui components
- 🌓 Dark/Light theme support
- 📧 Thread-based email browsing
- 🔍 Real-time search
- 👤 Author profiles with activity tracking
- ⚙️ Configurable API endpoint
- 📱 Responsive design

## Tech Stack

- React 19
- TypeScript
- Vite
- Tailwind CSS
- shadcn/ui
- React Router
- next-themes

## Getting Started

### Prerequisites

- Node.js 18+
- npm or yarn

### Installation

1. Install dependencies:
```bash
npm install
```

2. Copy the environment template:
```bash
cp .env.example .env
```

3. Update `.env` with your API server URL (default: `http://localhost:8000`)

4. Start the development server:
```bash
npm run dev
```

The app will be available at `http://localhost:5173`

### Build for Production

```bash
npm run build
```

The built files will be in the `dist/` directory.

### Preview Production Build

```bash
npm run preview
```

## Configuration

- **API Endpoint**: Configure in Settings dialog or via `VITE_API_URL` environment variable
- **Theme**: Toggle between light/dark modes via the theme button
- **Mailing List**: Select your preferred mailing list in the Settings dialog

## Project Structure

```
src/
├── components/         # React components
│   ├── ui/            # shadcn/ui components
│   ├── EmailItem.tsx  # Email display with collapse
│   ├── ThreadList.tsx # Thread list sidebar
│   ├── ThreadView.tsx # Thread detail view
│   └── ...
├── contexts/          # React contexts
├── lib/              # Utilities and API client
├── pages/            # Page components
├── types/            # TypeScript types
└── utils/            # Helper functions
```

## License

MIT
