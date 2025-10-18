# DeltaBrush

A 3D modelling software built with a Rust core (compiled to WebAssembly) and a Three.js web frontend.

## 🏗️ Project Structure

```
DeltaBrush/
├── src/
│   └── lib.rs           # Rust core with 3D geometry operations
├── www/
│   ├── index.html       # Main HTML file
│   ├── index.js         # Three.js integration and UI logic
│   └── style.css        # Styles
├── Cargo.toml           # Rust dependencies
├── package.json         # Node.js dependencies and build scripts
├── vite.config.js       # Vite configuration
└── README.md
```

## 🚀 Features

- **Rust Core**: High-performance geometry operations compiled to WebAssembly
  - 3D vector math (Vec3)
  - Mesh creation and manipulation
  - Procedural geometry generation (cubes, etc.)

- **Three.js Frontend**: Modern 3D rendering
  - Interactive 3D viewport with orbit controls
  - Real-time scene statistics
  - Procedural mesh creation using Rust backend

## 📋 Prerequisites

Before you begin, ensure you have the following installed:

1. **Rust** (latest stable version)
   - Install from [https://rustup.rs/](https://rustup.rs/)
   
2. **wasm-pack** (for building Rust to WebAssembly)
   ```powershell
   cargo install wasm-pack
   ```

3. **Node.js** (v18 or later)
   - Download from [https://nodejs.org/](https://nodejs.org/)

## 🛠️ Setup Instructions

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

## 🎮 Usage

Once the application is running:

1. **Create Cube**: Click the "Create Cube" button to add a new cube to the scene
   - Each cube is generated using Rust and positioned randomly
   - Cubes have random colors

2. **Clear Scene**: Remove all objects from the scene

3. **Navigate the 3D View**:
   - **Left Mouse**: Rotate camera
   - **Right Mouse**: Pan camera
   - **Mouse Wheel**: Zoom in/out

4. **Scene Info Panel**: View real-time statistics
   - Number of objects
   - Total vertex count
   - Total triangle count

## 🔧 Development

### Project Architecture

The project uses a hybrid architecture:

1. **Rust Core** (`src/lib.rs`):
   - Compiles to WebAssembly
   - Handles geometry calculations and mesh generation
   - Exposed to JavaScript via `wasm-bindgen`

2. **JavaScript Frontend** (`www/index.js`):
   - Imports Rust WASM module
   - Uses Three.js for rendering
   - Handles UI interactions and scene management

### Adding New Features

To add new geometry types:

1. Add Rust structs/functions in `src/lib.rs`
2. Mark them with `#[wasm_bindgen]` to expose to JavaScript
3. Rebuild the WASM module: `npm run build:wasm`
4. Use the new functions in `www/index.js`

### Building for Production

```powershell
npm run build
```

The built files will be in the `dist/` directory and can be deployed to any static hosting service.

## 📦 Dependencies

### Rust Dependencies
- `wasm-bindgen`: Rust/JavaScript interop
- `serde`: Serialization framework
- `web-sys`: Web API bindings

### JavaScript Dependencies
- `three`: 3D rendering library
- `vite`: Build tool and dev server

## 🐛 Troubleshooting

**Issue**: `wasm-pack not found`
- **Solution**: Install wasm-pack using `cargo install wasm-pack`

**Issue**: Build errors related to Rust
- **Solution**: Ensure Rust is up to date: `rustup update`

**Issue**: Port 3000 already in use
- **Solution**: Change the port in `vite.config.js` or stop the process using port 3000

## 📝 License

This project is open source and available under the MIT License.

## 🚀 Future Enhancements

- [ ] Additional primitive shapes (sphere, cylinder, etc.)
- [ ] Mesh editing tools (extrude, subdivide, etc.)
- [ ] Import/export 3D file formats (OBJ, STL, etc.)
- [ ] Material editor
- [ ] Transform tools (move, rotate, scale)
- [ ] Undo/redo functionality
- [ ] Save/load project files

## 🤝 Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.
