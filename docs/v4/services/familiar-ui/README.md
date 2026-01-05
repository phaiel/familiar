# Familiar UI

A modern, multimodal chat interface for the Heddle Classification Engine.
Built with Next.js, shadcn/ui, and prompt-kit.

## Features

- **Multimodal Input**: Text, Images, Audio, Documents.
- **Rich Rendering**: Displays Block Kit responses natively.
- **Dev Mode**: Toggle to reveal detailed Heddle classification metrics and pipeline status.
- **Real-time**: WebSocket streaming for job progress.

## Tech Stack

- **Framework**: Next.js 14 (App Router)
- **UI**: Tailwind CSS, shadcn/ui, prompt-kit
- **Icons**: Lucide React
- **State**: React Hooks (useChat)

## Getting Started

1. **Install dependencies**:
   ```bash
   npm install
   ```

2. **Run development server**:
   ```bash
   npm run dev
   ```

3. **Open browser**:
   Navigate to [http://localhost:3000](http://localhost:3000).

   Note: Ensure `familiar-api` is running on port 3001.

## Environment Variables

- `NEXT_PUBLIC_API_URL`: URL of the Familiar API (default: `/api` via proxy)
- `NEXT_PUBLIC_WS_URL`: WebSocket URL (default: `ws://localhost:3001`)

## Integration

This UI communicates with `familiar-api` via:
- `POST /api/weave`: Sends text/blocks.
- `WS /api/jobs/{id}/ws`: Streams progress and results.
