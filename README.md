# DeltaBrush

A 3D modelling software built with a Rust core (compiled to WebAssembly) and a Three.js web frontend.

## Prerequisites

Before you begin, ensure you have the following installed:

1. **Rust** (latest stable version)
   - Install from [https://rustup.rs/](https://rustup.rs/)
   
2. **wasm-pack** (for building Rust to WebAssembly)
   ```powershell
   cargo install wasm-pack
   ```

3. **Node.js** (v18 or later)
   - Download from [https://nodejs.org/](https://nodejs.org/)

## Setup Instructions

### 1. Install Dependencies

```powershell
npm install
```

### 2. Build the Project

Build both the Rust WebAssembly module and the web frontend:

```powershell
npm run build
```

Or build them separately:

```powershell
# Build Rust to WebAssembly
npm run build:wasm

# Build web frontend
npm run build:web
```

### 3. Run Development Server

Start the development server with hot-reloading:

```powershell
npm run dev
```

This will:
1. Build the Rust code to WebAssembly
2. Start Vite dev server
3. Open your browser at `http://localhost:3000`

## Development

### Building for Production

```powershell
npm run build
```

The built files will be in the `dist/` directory and can be deployed to any static hosting service.

## License

This project is proprietary. All rights reserved. Unauthorized copying, distribution, or modification is prohibited.