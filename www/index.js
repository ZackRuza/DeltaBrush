import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import init, { Scene as RustScene } from '../pkg/deltabrush.js';

class DeltaBrush {
    constructor() {
        this.scene = null;
        this.camera = null;
        this.renderer = null;
        this.controls = null;
        this.rustScene = null;
        this.threeObjects = new Map(); // Maps Rust object IDs to Three.js objects
        this.wasmInitialized = false;
    }

    async init() {
        // Initialize WASM
        await init();
        this.wasmInitialized = true;
        console.log('WASM initialized');

        // Create Rust scene
        this.rustScene = new RustScene();

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

        // Create cube in Rust scene
        const position = [
            (Math.random() - 0.5) * 4,
            (Math.random() - 0.5) * 4,
            (Math.random() - 0.5) * 4
        ];
        this.rustScene.add_cube(2.0, position);
    }

    clearScene() {
        // Clear Rust scene
        this.rustScene.clear();
    }

    syncScene() {
        if (!this.rustScene.is_dirty()) {
            return;
        }

        // Get all objects from Rust
        const sceneData = this.rustScene.get_scene_data();
        const currentIds = new Set();

        // Update or create Three.js objects
        for (const obj of sceneData) {
            currentIds.add(obj.id);

            if (!this.threeObjects.has(obj.id)) {
                // Create new Three.js object
                this.createThreeObject(obj);
            } else {
                // Update existing object
                this.updateThreeObject(obj);
            }
        }

        // Remove Three.js objects that no longer exist in Rust
        for (const [id, threeObj] of this.threeObjects.entries()) {
            if (!currentIds.has(id)) {
                this.scene.remove(threeObj);
                threeObj.geometry.dispose();
                threeObj.material.dispose();
                this.threeObjects.delete(id);
            }
        }

        this.rustScene.clear_dirty();
        this.updateInfo();
    }

    createThreeObject(rustObject) {
        const geometry = new THREE.BufferGeometry();
        const vertices = new Float32Array(rustObject.mesh_data.vertices);
        const indices = new Uint32Array(rustObject.mesh_data.indices);

        geometry.setAttribute('position', new THREE.BufferAttribute(vertices, 3));
        geometry.setIndex(new THREE.BufferAttribute(indices, 1));
        geometry.computeVertexNormals();

        const material = new THREE.MeshStandardMaterial({
            color: new THREE.Color(
                rustObject.material.color[0],
                rustObject.material.color[1],
                rustObject.material.color[2]
            ),
            metalness: rustObject.material.metalness,
            roughness: rustObject.material.roughness,
        });

        const mesh = new THREE.Mesh(geometry, material);
        this.updateThreeObjectTransform(mesh, rustObject.transform);

        this.scene.add(mesh);
        this.threeObjects.set(rustObject.id, mesh);
    }

    updateThreeObject(rustObject) {
        const threeObj = this.threeObjects.get(rustObject.id);
        if (threeObj) {
            this.updateThreeObjectTransform(threeObj, rustObject.transform);
        }
    }

    updateThreeObjectTransform(threeObj, transform) {
        threeObj.position.set(
            transform.position[0],
            transform.position[1],
            transform.position[2]
        );
        threeObj.quaternion.set(
            transform.rotation[0],
            transform.rotation[1],
            transform.rotation[2],
            transform.rotation[3]
        );
        threeObj.scale.set(
            transform.scale[0],
            transform.scale[1],
            transform.scale[2]
        );
    }

    updateInfo() {
        let totalVertices = 0;
        let totalTriangles = 0;

        this.threeObjects.forEach(mesh => {
            totalVertices += mesh.geometry.attributes.position.count;
            totalTriangles += mesh.geometry.index.count / 3;
        });

        document.getElementById('object-count').textContent = this.rustScene.object_count();
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
        
        // Sync Rust scene with Three.js scene
        this.syncScene();
        
        this.controls.update();
        this.renderer.render(this.scene, this.camera);
    }
}

// Initialize the app
const app = new DeltaBrush();
app.init().catch(console.error);
