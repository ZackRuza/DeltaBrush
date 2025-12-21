import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { EffectComposer } from 'three/examples/jsm/postprocessing/EffectComposer.js';
import { RenderPass } from 'three/examples/jsm/postprocessing/RenderPass.js';
import { OutlinePass } from 'three/examples/jsm/postprocessing/OutlinePass.js';
import { ShaderPass } from 'three/examples/jsm/postprocessing/ShaderPass.js';
import { FXAAShader } from 'three/examples/jsm/shaders/FXAAShader.js';
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
        this.meshCache = new Map(); // mesh_id -> THREE.BufferGeometry
        
        // Post-processing
        this.composer = null;
        this.outlinePass = null;
        
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
        this.scene.background = new THREE.Color(0x6a6a6a);

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
            antialias: true,
            powerPreference: "high-performance"
        });
        this.renderer.setSize(canvas.clientWidth, canvas.clientHeight);
        this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2)); // Cap at 2x for performance

        // Post-processing setup with high-quality settings
        const renderTarget = new THREE.WebGLRenderTarget(
            canvas.clientWidth * window.devicePixelRatio,
            canvas.clientHeight * window.devicePixelRatio,
            {
                minFilter: THREE.LinearFilter,
                magFilter: THREE.LinearFilter,
                format: THREE.RGBAFormat,
                stencilBuffer: false,
                samples: 8 // Enable MSAA (Multi-Sample Anti-Aliasing) for better quality
            }
        );
        
        this.composer = new EffectComposer(this.renderer, renderTarget);
        
        // Render pass - renders the scene normally
        const renderPass = new RenderPass(this.scene, this.camera);
        this.composer.addPass(renderPass);
        
        // Outline pass - adds outline effect to selected objects
        const pixelRatio = this.renderer.getPixelRatio();
        this.outlinePass = new OutlinePass(
            new THREE.Vector2(canvas.clientWidth * pixelRatio, canvas.clientHeight * pixelRatio),
            this.scene,
            this.camera
        );
        this.outlinePass.edgeStrength = 5.0; // Increased from 3.0 for thicker outline
        this.outlinePass.edgeGlow = 0.0; // No glow
        this.outlinePass.edgeThickness = 2.0; // Increased from 1.0 for thicker edges
        this.outlinePass.pulsePeriod = 0; // No pulsing animation
        this.outlinePass.visibleEdgeColor.set('#ffffff'); // White outline
        this.outlinePass.hiddenEdgeColor.set('#ffffff'); // White outline even when hidden
        this.outlinePass.renderToScreen = false; // Let FXAA be the final pass
        this.composer.addPass(this.outlinePass);
        
        // FXAA pass - anti-aliasing for post-processing (with higher quality settings)
        const fxaaPass = new ShaderPass(FXAAShader);
        fxaaPass.material.uniforms['resolution'].value.x = 1 / (canvas.clientWidth * pixelRatio);
        fxaaPass.material.uniforms['resolution'].value.y = 1 / (canvas.clientHeight * pixelRatio);
        this.composer.addPass(fxaaPass);

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
        // Ambient light - provides base illumination
        const ambientLight = new THREE.AmbientLight(0xffffff, 0.5);
        this.scene.add(ambientLight);

        // Key light (main directional light)
        const keyLight = new THREE.DirectionalLight(0xffffff, 1.0);
        keyLight.position.set(5, 10, 5);
        this.scene.add(keyLight);

        // Fill light (softer, from opposite side)
        const fillLight = new THREE.DirectionalLight(0xffffff, 0.5);
        fillLight.position.set(-5, 5, -5);
        this.scene.add(fillLight);

        // Rim light (from behind/side for edge definition)
        const rimLight = new THREE.DirectionalLight(0xffffff, 0.3);
        rimLight.position.set(0, 5, -10);
        this.scene.add(rimLight);
    }

    setupEventListeners() {
        document.getElementById("subbtn-1").addEventListener('click', () => {
            this.createCube();
        });

        document.getElementById("subbtn-2").addEventListener('click', () => {
            this.createSphere();
        });

        document.getElementById("btn-2").addEventListener('click', () => {
            this.clearScene();
        });

        document.getElementById("btn-1").addEventListener("click", () => {
            const subOptions = document.getElementById("subbuttons");
            subOptions.classList.toggle("show");
        });

        document.getElementById('subbtn-3').addEventListener('click', () => {
            document.getElementById('file-upload').click(); // triggers file picker
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

        // Keyboard shortcuts
        window.addEventListener('keydown', (event) => {
            this.onKeyDown(event);
        });

    }

    onKeyDown(event) {
        // Ctrl+Space: move selection up the hierarchy
        if (event.ctrlKey && event.code === 'Space') {
            event.preventDefault();

            if (!this.rustScene) return;

            const didSelectParent = this.rustScene.select_parent();
            if (didSelectParent) {
                document.getElementById('edit-mode').textContent = 'On';
            }
        }
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

    createPlane() {
        if (!this.wasmInitialized) {
            console.error('WASM not initialized');
            return;
        }

        // Create plane in Rust scene
        const position = [
            (Math.random() - 0.5) * 4,
            0.0, // Keep planes at y=0 for easier viewing
            (Math.random() - 0.5) * 4
        ];
        this.rustScene.add_plane(3.0, position);
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
                
                // Dispose of all meshes and materials in the group
                threeObj.traverse((child) => {
                    if (child.geometry) child.geometry.dispose();
                    if (child.material) child.material.dispose();
                });
                
                this.threeObjects.delete(id);
            }
        }

        // Update selection highlights for all objects
        this.updateAllSelectionHighlights(sceneData);

        this.rustScene.clear_dirty();
        this.updateInfo();
    }

    // Get or create geometry for a mesh_id
    getGeometry(meshId) {
        if (!this.meshCache.has(meshId)) {
            // Request mesh data from Rust
            const meshData = this.rustScene.get_mesh_data(meshId);
            
            const geometry = new THREE.BufferGeometry();
            const vertices = new Float32Array(meshData.vertex_coords);
            const indices = new Uint32Array(meshData.face_indices);
            
            geometry.setAttribute('position', new THREE.BufferAttribute(vertices, 3));
            geometry.setIndex(new THREE.BufferAttribute(indices, 1));
            geometry.computeVertexNormals();
            
            this.meshCache.set(meshId, geometry);
        }
        
        return this.meshCache.get(meshId);
    }

    createThreeObject(renderInstance) {
        const geometry = this.getGeometry(renderInstance.mesh_id);

        // TODO: for now, use default color
        const baseColor = new THREE.Color(0x808080);

        // Create front-facing material (opaque)
        const frontMaterial = new THREE.MeshStandardMaterial({
            color: baseColor,
            metalness: 0.5,
            roughness: 0.5,
            side: THREE.FrontSide,
            flatShading: true,
        });

        // Create back-facing material (translucent)
        const backMaterial = new THREE.MeshStandardMaterial({
            color: baseColor,
            metalness: 0.5,
            roughness: 0.5,
            side: THREE.BackSide,
            transparent: true,
            opacity: 0.3,
            flatShading: true,
        });

        // Create a group to hold both meshes
        const group = new THREE.Group();
        
        const frontMesh = new THREE.Mesh(geometry, frontMaterial);
        const backMesh = new THREE.Mesh(geometry, backMaterial);
        
        group.add(frontMesh);
        group.add(backMesh);
        
        this.updateThreeObjectTransform(group, renderInstance.transform);

        this.scene.add(group);
        this.threeObjects.set(renderInstance.id, group);
    }

    updateThreeObject(renderInstance) {
        const threeObj = this.threeObjects.get(renderInstance.id);
        if (threeObj) {
            this.updateThreeObjectTransform(threeObj, renderInstance.transform);
        }
    }

    updateAllSelectionHighlights(sceneData) {
        // Collect all selected objects for the outline pass
        const selectedMeshes = [];
        
        for (const renderInstance of sceneData) {
            const group = this.threeObjects.get(renderInstance.id);
            if (group && renderInstance.is_selected) {
                // Add both front and back meshes to selection
                selectedMeshes.push(group.children[0]); // front mesh
                selectedMeshes.push(group.children[1]); // back mesh
                
                // Apply visual highlighting (lighten color)
                this.applyVisualHighlight(group);
            } else if (group) {
                // Remove visual highlighting
                this.removeVisualHighlight(group);
            }
        }
        
        // Update outline pass with all selected objects at once
        this.outlinePass.selectedObjects = selectedMeshes;
    }

    applyVisualHighlight(group) {
        if (!group.userData.originalMaterial) {
            const frontMesh = group.children[0];
            const backMesh = group.children[1];
            
            // Store original material properties
            group.userData.originalMaterial = {
                frontColor: frontMesh.material.color.clone(),
                frontOpacity: frontMesh.material.opacity,
                frontTransparent: frontMesh.material.transparent,
                backColor: backMesh.material.color.clone(),
                backOpacity: backMesh.material.opacity,
            };
        }
        
        const frontMesh = group.children[0];
        const backMesh = group.children[1];
        
        // Make both meshes lighter and more translucent
        frontMesh.material.color.copy(group.userData.originalMaterial.frontColor).multiplyScalar(1.3);
        frontMesh.material.transparent = true;
        frontMesh.material.opacity = 0.7;
        
        backMesh.material.color.copy(group.userData.originalMaterial.backColor).multiplyScalar(1.3);
        backMesh.material.opacity = 0.2;
    }

    removeVisualHighlight(group) {
        if (group.userData.originalMaterial) {
            const frontMesh = group.children[0];
            const backMesh = group.children[1];
            
            // Restore original material properties
            frontMesh.material.color.copy(group.userData.originalMaterial.frontColor);
            frontMesh.material.opacity = group.userData.originalMaterial.frontOpacity;
            frontMesh.material.transparent = group.userData.originalMaterial.frontTransparent;
            
            backMesh.material.color.copy(group.userData.originalMaterial.backColor);
            backMesh.material.opacity = group.userData.originalMaterial.backOpacity;
            
            delete group.userData.originalMaterial;
        }
    }

    updateThreeObjectTransform(threeObj, transform) {
        threeObj.position.set(
            transform.translation[0],
            transform.translation[1],
            transform.translation[2]
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
            
            // Select the hit object using its edge UUID path
            if (hitResult.selection_path !== undefined) {
                this.rustScene.select_by_edge_path(hitResult.selection_path);
                console.log(`Selected object at edge path: [${hitResult.selection_path.join(', ')}]`);
                document.getElementById('selected-object').textContent = `Object ${hitResult.object_id}`;
                document.getElementById('edit-mode').textContent = 'On';
            }
        } else {
            console.log('No object hit');
            this.hideHitMarker();
            this.rustScene.deselect();
            document.getElementById('selected-object').textContent = 'None';
            document.getElementById('edit-mode').textContent = 'Off';
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



    onWindowResize() {
        const canvas = document.getElementById('canvas');
        const width = canvas.clientWidth;
        const height = canvas.clientHeight;
        const pixelRatio = this.renderer.getPixelRatio();
        
        this.camera.aspect = width / height;
        this.camera.updateProjectionMatrix();
        this.renderer.setSize(width, height);
        
        // Update composer and render target size
        this.composer.setSize(width, height);
        
        // Update the render target with high-quality settings
        const renderTarget = new THREE.WebGLRenderTarget(
            width * pixelRatio,
            height * pixelRatio,
            {
                minFilter: THREE.LinearFilter,
                magFilter: THREE.LinearFilter,
                format: THREE.RGBAFormat,
                stencilBuffer: false,
                samples: 8 // Maintain MSAA quality
            }
        );
        this.composer.renderTarget1 = renderTarget;
        this.composer.renderTarget2 = renderTarget.clone();
        
        // Update outline pass resolution (use pixel ratio for higher quality)
        this.outlinePass.resolution.set(width * pixelRatio, height * pixelRatio);
        
        // Update FXAA shader resolution
        const fxaaPass = this.composer.passes[2]; // FXAA is the 3rd pass
        if (fxaaPass && fxaaPass.material.uniforms['resolution']) {
            fxaaPass.material.uniforms['resolution'].value.x = 1 / (width * pixelRatio);
            fxaaPass.material.uniforms['resolution'].value.y = 1 / (height * pixelRatio);
        }
    }

    animate() {
        requestAnimationFrame(() => this.animate());
        
        // Sync Rust scene with Three.js scene
        this.syncScene();
        
        this.controls.update();
        this.composer.render(); // Use composer instead of renderer
    }
}

// Initialize the app
const app = new DeltaBrush();
app.init().catch(console.error);
