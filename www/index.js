import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import init, { SceneAPI as RustScene } from '../pkg/deltabrush.js';

class DeltaBrush {
    constructor() {
        this.scene = null;
        this.camera = null;
        this.renderer = null;
        this.controls = null;
        this.rustScene = null;
        this.threeObjects = new Map(); // Maps Rust object IDs to Three.js objects
        this.wasmInitialized = false;
        
        // Mouse interaction state
        this.mouseDownPos = null;
        this.mouseUpPos = null;
        this.isDragging = false;
        
        // Hit visualization
        this.hitMarker = null;
        
        // Selection system
        this.selectedObjectId = null;
        this.highlightMeshes = new Map(); // Maps object IDs to highlight wireframes
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

        document.getElementById('create-sphere').addEventListener('click', () => {
            this.createSphere();
        });

        document.getElementById('clear-scene').addEventListener('click', () => {
            this.clearScene();
        });

        // Mouse click detection
        const canvas = this.renderer.domElement;
        
        canvas.addEventListener('mousedown', (event) => {
            this.onMouseDown(event);
        });

        canvas.addEventListener('mousemove', (event) => {
            this.onMouseMove(event);
        });

        canvas.addEventListener('mouseup', (event) => {
            this.onMouseUp(event);
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

    createSphere() {
        if (!this.wasmInitialized) {
            console.error('WASM not initialized');
            return;
        }

        // Create sphere in Rust scene
        const position = [
            (Math.random() - 0.5) * 4,
            (Math.random() - 0.5) * 4,
            (Math.random() - 0.5) * 4
        ];
        this.rustScene.add_sphere(1.0, position);
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
                
                // Also remove highlight if it exists
                this.removeHighlight(id);
            }
        }

        this.rustScene.clear_dirty();
        this.updateInfo();
    }

    createThreeObject(rustObject) {
        const geometry = new THREE.BufferGeometry();
        const vertices = new Float32Array(rustObject.mesh.vertex_coords);
        const indices = new Uint32Array(rustObject.mesh.face_indices);

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
        
        // Store object ID on the mesh for raycasting identification
        mesh.userData.objectId = rustObject.id;
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

    onMouseDown(event) {
        // Store the initial mouse position
        this.mouseDownPos = {
            x: event.clientX,
            y: event.clientY
        };
        this.isDragging = false;
    }

    onMouseMove(event) {
        // If mouse has moved significantly, consider it a drag
        if (this.mouseDownPos) {
            const dx = event.clientX - this.mouseDownPos.x;
            const dy = event.clientY - this.mouseDownPos.y;
            const distance = Math.sqrt(dx * dx + dy * dy);
            
            // Consider it a drag if mouse moved more than 5 pixels
            if (distance > 5) {
                this.isDragging = true;
            }
        }
    }

    onMouseUp(event) {
        // Only process as a click if we didn't drag
        if (!this.isDragging && this.mouseDownPos) {
            this.mouseUpPos = {
                x: event.clientX,
                y: event.clientY
            };
            
            // Convert to normalized device coordinates
            const canvas = this.renderer.domElement;
            const rect = canvas.getBoundingClientRect();
            const x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
            const y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
            
            console.log('Click detected at screen:', event.clientX, event.clientY);
            console.log('NDC coordinates:', x.toFixed(3), y.toFixed(3));
            
            this.handleClick(x, y);
        }
        
        // Reset state
        this.mouseDownPos = null;
        this.mouseUpPos = null;
        this.isDragging = false;
    }

    handleClick(ndcX, ndcY) {
        const raycaster = new THREE.Raycaster();
        const mouse = new THREE.Vector2(ndcX, ndcY);
        raycaster.setFromCamera(mouse, this.camera);

        // Extract ray origin (camera position) and direction
        const origin = raycaster.ray.origin;
        const direction = raycaster.ray.direction;
        
        console.log('Ray origin (camera position):', origin);
        console.log('Ray direction:', direction);
        
        // Send ray data to Rust for raycasting
        const hitResult = this.rustScene.raycast_closest_hit(
            [origin.x, origin.y, origin.z],
            [direction.x, direction.y, direction.z]
        );
        
        console.log('Raw hitResult:', hitResult);
        console.log('hitResult type:', typeof hitResult);
        
        if (hitResult && hitResult !== null) {
            console.log('Hit data structure:', hitResult);
            console.log('Position field:', hitResult.position);
            console.log('Object ID field:', hitResult.object_id);
            
            if (hitResult.position) {
                this.showHitMarker(hitResult.position.x, hitResult.position.y, hitResult.position.z);
            }
            
            // Select the hit object (entering edit mode)
            if (hitResult.object_id !== undefined) {
                const objectId = hitResult.object_id;
                
                // Check if clicking the same object
                if (this.selectedObjectId === objectId) {
                    console.log(`Object ${objectId} is already selected (in edit mode)`);
                    return;
                }
                
                // Select object (this enters "edit mode" - just means it's selected)
                this.selectObject(objectId);
                console.log(`Object ${objectId} selected (edit mode active)`);
                document.getElementById('edit-mode').textContent = 'On';
            }
        } else {
            console.log('No object hit');
            this.hideHitMarker();
            this.clearSelection();
        }
    }

    showHitMarker(x, y, z) {
        // Remove existing marker if present
        if (this.hitMarker) {
            this.scene.remove(this.hitMarker);
            this.hitMarker.geometry.dispose();
            this.hitMarker.material.dispose();
        }

        // Create a small sphere to mark the hit position
        const geometry = new THREE.SphereGeometry(0.1, 16, 16);
        const material = new THREE.MeshBasicMaterial({ 
            color: 0xff0000,
            transparent: true,
            opacity: 0.8
        });
        this.hitMarker = new THREE.Mesh(geometry, material);
        this.hitMarker.position.set(x, y, z);
        
        this.scene.add(this.hitMarker);
    }

    hideHitMarker() {
        if (this.hitMarker) {
            this.scene.remove(this.hitMarker);
            this.hitMarker.geometry.dispose();
            this.hitMarker.material.dispose();
            this.hitMarker = null;
        }
    }

    selectObject(objectId) {
        // Clear previous selection
        this.clearSelection();
        
        // Set new selection
        this.selectedObjectId = objectId;
        
        // Create highlight for the selected object
        this.createHighlight(objectId);
        
        // Update UI
        document.getElementById('selected-object').textContent = objectId;
    }

    clearSelection() {
        if (this.selectedObjectId !== null) {
            // Deselect (exit "edit mode" - just means nothing is selected)
            console.log(`Deselecting object ${this.selectedObjectId} (exiting edit mode)`);
            
            this.removeHighlight(this.selectedObjectId);
            this.selectedObjectId = null;
            
            // Update UI
            document.getElementById('selected-object').textContent = 'None';
            document.getElementById('edit-mode').textContent = 'Off';
        }
    }

    createHighlight(objectId) {
        const mesh = this.threeObjects.get(objectId);
        if (!mesh) return;

        // Create wireframe geometry from the same geometry
        const wireframeGeometry = new THREE.WireframeGeometry(mesh.geometry);
        const wireframeMaterial = new THREE.LineBasicMaterial({
            color: 0x00ff00,
            linewidth: 2,
            transparent: true,
            opacity: 0.8
        });

        const wireframe = new THREE.LineSegments(wireframeGeometry, wireframeMaterial);
        
        // Match the transform of the original mesh
        wireframe.position.copy(mesh.position);
        wireframe.rotation.copy(mesh.rotation);
        wireframe.scale.copy(mesh.scale);
        
        // Scale slightly larger to avoid z-fighting
        wireframe.scale.multiplyScalar(1.002);

        this.scene.add(wireframe);
        this.highlightMeshes.set(objectId, wireframe);
    }

    removeHighlight(objectId) {
        const highlight = this.highlightMeshes.get(objectId);
        if (highlight) {
            this.scene.remove(highlight);
            highlight.geometry.dispose();
            highlight.material.dispose();
            this.highlightMeshes.delete(objectId);
        }
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
