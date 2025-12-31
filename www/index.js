import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { OBJLoader } from 'three/examples/jsm/loaders/OBJLoader.js';
import { MTLLoader } from 'three/examples/jsm/loaders/MTLLoader.js';
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
        
        // Translation gnomon interaction
        this.translationGnomonGroup = null;
        this.translationGnomonMeshes = []; // Array of translation gnomon axis meshes
        this.hoveredTranslationAxis = null; // Currently hovered axis ('x_axis', 'y_axis', 'z_axis')
        this.draggingTranslationAxis = null; // Currently dragging axis
        this.translationDragDistance = null;
        this.translationDragStartPosition = null; // Starting position of the gnomon
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
        await this.setupTranslationGnomon();
        this.setupEventListeners();
        this.setupModelsPanel();
        this.animate();
    }

    resetCamera() {
        if (!this.camera || !this.controls) return;

        this.camera.position.set(5, 5, 5);

        this.controls.target.set(0, 0, 0);

        this.controls.saveState();

        this.controls.update();
    }

    handleFileUpload(event) {
        const file = event.target.files[0];
        if (!file) return;

        if (!this.wasmInitialized || !this.rustScene) {
            console.error('Scene not ready');
            event.target.value = '';
            return;
        }

        file.text().then(text => {
            const meshId = this.rustScene.import_obj(file.name, text);
            this.addModelToPanel(file.name, meshId);
        }).finally(() => {
            event.target.value = '';
        });
    }
    
    openFilePicker() {
        document.getElementById('file-upload').click();
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

    async setupTranslationGnomon() {
        try {
            console.log('Loading translation gnomon...');
            
            // Load shared gizmo materials
            const mtlLoader = new MTLLoader();
            const materials = await mtlLoader.loadAsync('/models/gizmo.mtl');
            materials.preload();
            
            // Load OBJ with materials
            const objLoader = new OBJLoader();
            objLoader.setMaterials(materials);
            const gnomon = await objLoader.loadAsync('/models/translation_gnomon.obj');
            
            // Store reference to translation gnomon group
            this.translationGnomonGroup = gnomon;
            this.translationGnomonGroup.name = 'translation_gnomon';
            
            // Store references to each axis mesh for raycasting
            this.translationGnomonMeshes = [];
            gnomon.traverse((child) => {
                if (child instanceof THREE.Mesh) {
                    child.userData.isTranslationAxis = true;
                    child.userData.axisName = child.name; // 'x_axis', 'y_axis', 'z_axis'
                    this.translationGnomonMeshes.push(child);
                }
            });
            
            // Add the gnomon to the scene
            this.scene.add(gnomon);
            
            console.log('Translation gnomon loaded with', this.translationGnomonMeshes.length, 'axis meshes');
        } catch (error) {
            console.error('Failed to load translation gnomon:', error);
        }
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

        document.getElementById('reset-camera-btn').addEventListener('click', () => {
            this.resetCamera();
        });

        document.getElementById('file-upload').addEventListener('change', (e) => this.handleFileUpload(e));

        document.getElementById('subbtn-3').addEventListener('click', () => this.openFilePicker());

        document.getElementById('upload-mesh-btn').addEventListener('click', () => this.openFilePicker());

        document.getElementById('models-upload-btn').addEventListener('click', () => this.openFilePicker());

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


        const tabs = document.querySelectorAll('.tab');
        const panels = document.querySelectorAll('.tab-content');


        tabs.forEach(tab => {
            tab.addEventListener('click', () => {

                tabs.forEach(t => t.classList.remove('active'));
                panels.forEach(p => p.classList.remove('active'));

                tab.classList.add('active');

                const targetId = tab.dataset.tab;
                document.getElementById(targetId).classList.add('active');
            });
        });

        const app = document.getElementById("app");
        const handle = document.querySelector(".resize-handle");

        let isResizing = false;

        handle.addEventListener("mousedown", () => {
            isResizing = true;
        });

        window.addEventListener("mousemove", (e) => {
            if (!isResizing) return;

            const appRect = app.getBoundingClientRect();
            const newAsideWidth = appRect.right - e.clientX;

            if (newAsideWidth < 250 || newAsideWidth > 500) return;

            app.style.gridTemplateColumns = `1fr 4px ${newAsideWidth}px`;
        });

        window.addEventListener("mouseup", () => {
            isResizing = false;
        });

    }

    setupModelsPanel() {
        this.modelsListContainer = document.getElementById("models-list");
    }


    addModelToPanel(name, meshId) {
        if (!this.modelsListContainer) return;

        const modelItem = document.createElement("div");
        modelItem.classList.add("model-item");
        modelItem.dataset.meshId = meshId;

        const modelImage = document.createElement("div");
        modelImage.classList.add("model-image");

        const nameSpan = document.createElement("span");
        nameSpan.classList.add("model-name");
        nameSpan.textContent = name || "(unnamed)";

        const actions = document.createElement("div");
        actions.classList.add("model-actions");

        const settingsBtn = document.createElement("button");
        settingsBtn.textContent = "⚙️";
        settingsBtn.classList.add("model-action-btn");

        const deleteBtn = document.createElement("button");
        deleteBtn.textContent = "❌";
        deleteBtn.classList.add("model-action-btn");
        deleteBtn.addEventListener("click", () => {
            modelItem.remove();
            this.updateModelList();
        });

        actions.appendChild(settingsBtn);
        actions.appendChild(deleteBtn);

        modelItem.appendChild(modelImage);
        modelItem.appendChild(nameSpan);
        modelItem.appendChild(actions);

        this.modelsListContainer.appendChild(modelItem);
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

        // Add cube to models list
        this.rustScene.add_cube(2.0);
        this.updateModelList();
    }

    createSphere() {
        if (!this.wasmInitialized) {
            console.error('WASM not initialized');
            return;
        }

        // Add sphere to models list
        this.rustScene.add_sphere(1.0);
        this.updateModelList();
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
        this.updateModelList();
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

    updateModelList() {
        const models = this.rustScene.get_model_list(); 
        if (!models) return;

        if (this.modelsListContainer) this.modelsListContainer.innerHTML = '';

        for (const [meshId, name] of models) {
            this.addModelToPanel(name, meshId);
        }
    }


    onMouseDown(event) {
        // Store the initial mouse position
        this.mouseDownPos = {
            x: event.clientX,
            y: event.clientY
        };
        this.isDragging = false;
        
        // Check if clicking on translation gnomon axis
        if (this.hoveredTranslationAxis) {
            this.draggingTranslationAxis = this.hoveredTranslationAxis;
            this.translationDragStart = { x: event.clientX, y: event.clientY };
            this.translationDragStartPosition = this.translationGnomonGroup.position.clone();
            this.translationDragDistance = null; // Will be set on first drag frame
            
            // Disable orbit controls while dragging gnomon
            if (this.controls) {
                this.controls.enabled = false;
            }
            
            // Set grabbing cursor
            this.renderer.domElement.style.cursor = 'grabbing';
        }
    }

    onMouseMove(event) {
        const canvas = this.renderer.domElement;
        const rect = canvas.getBoundingClientRect();
        
        // If dragging translation gnomon axis, handle the drag
        if (this.draggingTranslationAxis && this.translationDragStart) {
            this.isDragging = true;
            this.handleTranslationDrag(event);
            return;
        }
        
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
        
        // Check for translation gnomon hover (only when not dragging)
        if (!this.isDragging) {
            const x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
            const y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
            
            const raycaster = new THREE.Raycaster();
            raycaster.setFromCamera(new THREE.Vector2(x, y), this.camera);
            
            // Check intersection with translation gnomon meshes
            const intersects = raycaster.intersectObjects(this.translationGnomonMeshes, false);
            
            if (intersects.length > 0) {
                const hitMesh = intersects[0].object;
                const axisName = hitMesh.userData.axisName;
                
                if (this.hoveredTranslationAxis !== axisName) {
                    this.hoveredTranslationAxis = axisName;
                    canvas.style.cursor = 'pointer';
                }
            } else {
                if (this.hoveredTranslationAxis) {
                    this.hoveredTranslationAxis = null;
                    canvas.style.cursor = 'default';
                }
            }
        }
    }

    onMouseUp(event) {
        // End translation gnomon drag if active
        if (this.draggingTranslationAxis) {
            this.draggingTranslationAxis = null;
            this.translationDragStart = null;
            this.translationDragStartPosition = null;
            this.translationDragDistance = null;
            
            // Re-enable orbit controls
            if (this.controls) {
                this.controls.enabled = true;
            }
            
            // Reset cursor
            this.renderer.domElement.style.cursor = this.hoveredTranslationAxis ? 'pointer' : 'default';
        }
        
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

    handleTranslationDrag(event) {
        if (!this.draggingTranslationAxis || !this.translationDragStartPosition || !this.translationGnomonGroup) return;
        
        const canvas = this.renderer.domElement;
        const rect = canvas.getBoundingClientRect();
        
        // Convert world position to screen pixels
        const worldToScreen = (worldPos) => {
            const ndc = worldPos.clone().project(this.camera);
            return new THREE.Vector2(
                (ndc.x + 1) / 2 * rect.width,
                (1 - ndc.y) / 2 * rect.height
            );
        };
        
        // Get local axis direction based on which axis is being dragged
        const localAxisDir = new THREE.Vector3();
        switch (this.draggingTranslationAxis) {
            case 'x_axis': localAxisDir.set(1, 0, 0); break;
            case 'y_axis': localAxisDir.set(0, 1, 0); break;
            case 'z_axis': localAxisDir.set(0, 0, 1); break;
            default: return;
        }

        // Convert local axis to world space
        const worldAxisDir = localAxisDir.applyQuaternion(this.translationGnomonGroup.quaternion);

        // Calculate axis direction in screen space
        const originScreen = worldToScreen(this.translationDragStartPosition);
        const axisTipScreen = worldToScreen(this.translationDragStartPosition.clone().add(worldAxisDir));
        const axisScreenDir = axisTipScreen.sub(originScreen);

        // Skip if axis is nearly perpendicular to screen
        const axisLengthSquared = axisScreenDir.lengthSq();
        if (axisLengthSquared < 0.0001) return;

        // Get mouse position relative to gnomon origin (in screen pixels)
        const mouseFromOrigin = new THREE.Vector2(
            event.clientX - rect.left - originScreen.x,
            event.clientY - rect.top - originScreen.y
        );
        
        // Project mouse onto axis to get distance in world units
        const projectionScalar = mouseFromOrigin.dot(axisScreenDir) / axisLengthSquared;
        
        // Store initial projection on first frame
        if (this.translationDragDistance === null) {
            this.translationDragDistance = projectionScalar;
        }

        // Apply movement delta along world axis
        const deltaWorld = projectionScalar - this.translationDragDistance;
        this.translationGnomonGroup.position.copy(
            this.translationDragStartPosition.clone().add(worldAxisDir.multiplyScalar(deltaWorld))
        );
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
