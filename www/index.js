import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import init, { Mesh } from '../pkg/deltabrush.js';

class DeltaBrush {
    constructor() {
        this.scene = null;
        this.camera = null;
        this.renderer = null;
        this.controls = null;
        this.meshes = [];
        this.wasmInitialized = false;
    }

    async init() {
        // Initialize WASM
        await init();
        this.wasmInitialized = true;
        console.log('WASM initialized');

        // Setup Three.js scene
        this.setupScene();
        this.setupLights();
        this.setupEventListeners();
        this.animate();
    }

    setupScene() {
        const canvas = document.getElementById('canvas');
        
        // Scene
        this.scene = new THREE.Scene();
        this.scene.background = new THREE.Color(0x2a2a2a);

        // Camera
        this.camera = new THREE.PerspectiveCamera(
            75,
            canvas.clientWidth / canvas.clientHeight,
            0.1,
            1000
        );
        this.camera.position.set(5, 5, 5);

        // Renderer
        this.renderer = new THREE.WebGLRenderer({ 
            canvas: canvas,
            antialias: true 
        });
        this.renderer.setSize(canvas.clientWidth, canvas.clientHeight);
        this.renderer.setPixelRatio(window.devicePixelRatio);

        // Controls
        this.controls = new OrbitControls(this.camera, this.renderer.domElement);
        this.controls.enableDamping = true;
        this.controls.dampingFactor = 0.05;

        // Grid helper
        const gridHelper = new THREE.GridHelper(10, 10);
        this.scene.add(gridHelper);

        // Axes helper
        const axesHelper = new THREE.AxesHelper(5);
        this.scene.add(axesHelper);

        // Handle window resize
        window.addEventListener('resize', () => this.onWindowResize());
    }

    setupLights() {
        // Ambient light
        const ambientLight = new THREE.AmbientLight(0xffffff, 0.6);
        this.scene.add(ambientLight);

        // Directional light
        const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
        directionalLight.position.set(5, 10, 5);
        this.scene.add(directionalLight);
    }

    setupEventListeners() {
        document.getElementById('create-cube').addEventListener('click', () => {
            this.createCube();
        });

        document.getElementById('clear-scene').addEventListener('click', () => {
            this.clearScene();
        });
    }

    createCube() {
        if (!this.wasmInitialized) {
            console.error('WASM not initialized');
            return;
        }

        // Create mesh using Rust
        const rustMesh = Mesh.create_cube(2.0);
        const vertices = rustMesh.get_vertices_flat();
        const indices = rustMesh.get_indices();

        console.log(`Created cube with ${rustMesh.vertex_count()} vertices and ${rustMesh.triangle_count()} triangles`);

        // Create Three.js geometry
        const geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', new THREE.Float32BufferAttribute(vertices, 3));
        geometry.setIndex(indices);
        geometry.computeVertexNormals();

        // Create material
        const material = new THREE.MeshStandardMaterial({
            color: Math.random() * 0xffffff,
            metalness: 0.3,
            roughness: 0.4,
        });

        // Create mesh and add to scene
        const mesh = new THREE.Mesh(geometry, material);
        mesh.position.set(
            (Math.random() - 0.5) * 4,
            (Math.random() - 0.5) * 4,
            (Math.random() - 0.5) * 4
        );
        
        this.scene.add(mesh);
        this.meshes.push(mesh);

        this.updateInfo();
    }

    clearScene() {
        // Remove all meshes from scene
        this.meshes.forEach(mesh => {
            this.scene.remove(mesh);
            mesh.geometry.dispose();
            mesh.material.dispose();
        });
        this.meshes = [];
        this.updateInfo();
    }

    updateInfo() {
        let totalVertices = 0;
        let totalTriangles = 0;

        this.meshes.forEach(mesh => {
            totalVertices += mesh.geometry.attributes.position.count;
            totalTriangles += mesh.geometry.index.count / 3;
        });

        document.getElementById('object-count').textContent = this.meshes.length;
        document.getElementById('vertex-count').textContent = totalVertices;
        document.getElementById('triangle-count').textContent = totalTriangles;
    }

    onWindowResize() {
        const canvas = document.getElementById('canvas');
        this.camera.aspect = canvas.clientWidth / canvas.clientHeight;
        this.camera.updateProjectionMatrix();
        this.renderer.setSize(canvas.clientWidth, canvas.clientHeight);
    }

    animate() {
        requestAnimationFrame(() => this.animate());
        this.controls.update();
        this.renderer.render(this.scene, this.camera);
    }
}

// Initialize the app
const app = new DeltaBrush();
app.init().catch(console.error);
