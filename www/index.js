import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { OBJLoader } from 'three/examples/jsm/loaders/OBJLoader.js';
import { MTLLoader } from 'three/examples/jsm/loaders/MTLLoader.js';
import { EffectComposer } from 'three/examples/jsm/postprocessing/EffectComposer.js';
import { DieQuadShader } from './shaders/DieQuadShader.js';
import { RenderPass } from 'three/examples/jsm/postprocessing/RenderPass.js';
import { OutlinePass } from 'three/examples/jsm/postprocessing/OutlinePass.js';
import { ShaderPass } from 'three/examples/jsm/postprocessing/ShaderPass.js';
import { FXAAShader } from 'three/examples/jsm/shaders/FXAAShader.js';
import init, { SceneAPI as RustScene } from '../pkg/deltabrush.js';
import { ThreeMFLoader } from 'three/examples/jsm/Addons.js';

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
        
        // Rotation gnomon interaction
        this.rotationGnomonGroup = null;
        this.rotationGnomonMeshes = []; // Array of rotation gnomon plane meshes
        this.hoveredRotationPlane = null; // Currently hovered plane ('xy_plane', 'yz_plane', 'zx_plane')
        this.draggingRotationPlane = null; // Currently dragging plane
        this.rotationDragStartMouse = null; // Starting mouse position for rotation drag
        this.rotationDragStartQuaternion = null; // Starting quaternion of the gnomon
        this.rotationDragStartPosition = null; // Need starting position to determine mouse drag angle
        
        // Scale gnomon interaction
        this.scaleGnomonGroup = null;
        this.scaleGnomonAxes = null; // The axes group
        this.scaleGnomonAxisObjects = { x: null, y: null, z: null }; // Individual axis references
        this.scaleGnomonScale = new THREE.Vector3(1, 1, 1); // Logical scale for meshes
        this.scaleGnomonTips = []; // Array of tip objects { mesh, axis: 'x'|'y'|'z' }
        this.scaleGnomonMeshes = []; // Array of scale gnomon axis meshes (for raycasting)
        this.hoveredScaleAxis = null; // Currently hovered axis ('x_axis', 'y_axis', 'z_axis')
        this.draggingScaleAxis = null; // Currently dragging axis
        this.scaleDragDistance = null; // Starting scale of the gnomon
        this.scaleDragStartPosition = null; // Starting position of the gnomon
        
        // Pointed die gizmo for transformation visualization
        this.referencePointedDie = null; // Translucent die at origin
        this.transformedPointedDie = null; // Opaque die after transformation
        this.referencePointedDieRT = null; // Render target for reference die
        this.transformedPointedDieRT = null; // Render target for transformed die
        this.referencePointedDieScene = null; // Separate scene for reference die
        this.transformedPointedDieScene = null; // Separate scene for transformed die
        this.dieQuadScene = null; // Scene for screen-space quads
        this.dieQuadCamera = null; // Orthographic camera for quads
        
        // Resize handling
        this.resizeTimeout = null;
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
        //await this.setupTranslationGnomon();
        //await this.setupRotationGnomon();
        await this.setupScaleGnomon();
        await this.setupPointedDie();
        this.setupEventListeners();
        this.setupModelsPanel();
        
        // Setup resize observer for reliable container resize detection
        this.setupResizeObserver();
        
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
        // Use parent container dimensions for reliable sizing
        const container = canvas.parentElement;
        const width = container.clientWidth || window.innerWidth;
        const height = container.clientHeight || window.innerHeight;
        
        console.log('Setting up scene with dimensions:', width, 'x', height);
        
        // Scene
        this.scene = new THREE.Scene();
        this.scene.background = new THREE.Color(0x6a6a6a);

        // Camera
        this.camera = new THREE.PerspectiveCamera(
            75,
            width / height,
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
        // Pass false to prevent Three.js from setting CSS styles (which conflicts with our CSS)
        this.renderer.setSize(width, height, false);
        this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2)); // Cap at 2x for performance

        // Post-processing setup with high-quality settings
        const renderTarget = new THREE.WebGLRenderTarget(
            width * window.devicePixelRatio,
            height * window.devicePixelRatio,
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
            new THREE.Vector2(width * pixelRatio, height * pixelRatio),
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
        fxaaPass.material.uniforms['resolution'].value.x = 1 / (width * pixelRatio);
        fxaaPass.material.uniforms['resolution'].value.y = 1 / (height * pixelRatio);
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
    }

    setupResizeObserver() {
        const container = this.renderer.domElement.parentElement;
        
        this.resizeObserver = new ResizeObserver((entries) => {
            for (const entry of entries) {
                const { width, height } = entry.contentRect;
                
                if (width > 0 && height > 0) {
                    // Immediately update camera to prevent distortion
                    this.camera.aspect = width / height;
                    this.camera.updateProjectionMatrix();
                    
                    // Debounce expensive buffer resize operations
                    this.debouncedResize(width, height);
                }
            }
        });
        
        this.resizeObserver.observe(container);
    }
    
    debouncedResize(width, height) {
        // Clear existing timeout
        if (this.resizeTimeout) {
            clearTimeout(this.resizeTimeout);
        }
        
        // Wait 100ms after last resize event before applying expensive operations
        this.resizeTimeout = setTimeout(() => {
            this.applyResize(width, height);
        }, 100);
    }
    
    applyResize(width, height) {
        // Update renderer buffers (expensive operation, debounced)
        this.renderer.setSize(width, height, false);
        
        // Update post-processing
        this.composer.setSize(width, height);
        
        const pixelRatio = this.renderer.getPixelRatio();
        this.outlinePass.setSize(width * pixelRatio, height * pixelRatio);
        
        // Update FXAA resolution
        const fxaaPass = this.composer.passes[this.composer.passes.length - 1];
        if (fxaaPass?.material?.uniforms['resolution']) {
            fxaaPass.material.uniforms['resolution'].value.set(1 / (width * pixelRatio), 1 / (height * pixelRatio));
        }
        
        // Update pointed die render targets
        if (this.referencePointedDieRT) {
            this.referencePointedDieRT.setSize(width, height);
        }
        if (this.transformedPointedDieRT) {
            this.transformedPointedDieRT.setSize(width, height);
        }
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

    // Helper method to load a gnomon with shared logic
    async loadGnomon(filename, gnomonType, meshFilter = null) {
        try {
            console.log(`Loading ${gnomonType} gnomon...`);
            
            // Load shared gizmo materials
            const mtlLoader = new MTLLoader();
            const materials = await mtlLoader.loadAsync('/models/gizmo.mtl');
            materials.preload();
            
            // Load OBJ with materials
            const objLoader = new OBJLoader();
            objLoader.setMaterials(materials);
            const gnomon = await objLoader.loadAsync(`/models/${filename}`);
            gnomon.name = `${gnomonType}_gnomon`;
            
            // Collect meshes based on filter
            const meshes = [];
            gnomon.traverse((child) => {
                if (child instanceof THREE.Mesh) {
                    if (!meshFilter || meshFilter(child)) {
                        meshes.push(child);
                    }
                }
            });
            
            // Add to scene
            this.scene.add(gnomon);
            
            console.log(`${gnomonType} gnomon loaded with`, meshes.length, 'meshes');
            return { group: gnomon, meshes };
        } catch (error) {
            console.error(`Failed to load ${gnomonType} gnomon:`, error);
            return null;
        }
    }

    async setupTranslationGnomon() {
        const result = await this.loadGnomon('translation_gnomon.obj', 'translation', (child) => {
            child.userData.isTranslationAxis = true;
            child.userData.axisName = child.name;
            return true;
        });
        if (result) {
            this.translationGnomonGroup = result.group;
            this.translationGnomonMeshes = result.meshes;
        }
    }

    async setupRotationGnomon() {
        const result = await this.loadGnomon('rotation_gnomon.obj', 'rotation', (child) => {
            // Only include plane meshes, not axis cylinders
            if (child.name.includes('_plane')) {
                child.userData.isRotationPlane = true;
                child.userData.planeName = child.name;
                return true;
            }
            return false;
        });
        if (result) {
            this.rotationGnomonGroup = result.group;
            this.rotationGnomonMeshes = result.meshes;
        }
    }

    async setupScaleGnomon() {
        try {
            console.log('Loading scale gnomon...');
            
            // Load shared gizmo materials
            const mtlLoader = new MTLLoader();
            const materials = await mtlLoader.loadAsync('/models/gizmo.mtl');
            materials.preload();
            
            // Create main group for the scale gnomon
            this.scaleGnomonGroup = new THREE.Group();
            this.scaleGnomonGroup.name = 'scale_gnomon';
            
            // Build axes from a single axis mesh (gnomon_axis.obj) rotated into x/y/z
            const objLoader = new OBJLoader();
            objLoader.setMaterials(materials);
            const baseAxis = await objLoader.loadAsync('/models/gnomon_axis.obj'); // z-axis model

            // Group to hold the three axes
            const axesGroup = new THREE.Group();
            axesGroup.name = 'scale_gnomon_axes';

            const axisConfigs = [
                { axis: 'x', material: 'axis_x', rotation: [0, Math.PI / 2, 0] },   // rotate z→x (positive Y)
                { axis: 'y', material: 'axis_y', rotation: [-Math.PI / 2, 0, 0] },  // rotate z→y
                { axis: 'z', material: 'axis_z', rotation: [0, 0, 0] }              // keep z
            ];

            // Collect axis meshes for raycasting
            this.scaleGnomonMeshes = [];

            for (const config of axisConfigs) {
                const axisClone = baseAxis.clone(true);
                axisClone.name = `${config.axis}_axis_object`;

                // Apply rotation to align the axis
                axisClone.rotation.set(...config.rotation);

                // Apply material and tagging
                axisClone.traverse((child) => {
                    if (child instanceof THREE.Mesh) {
                        const mat = materials.materials[config.material];
                        if (mat) {
                            child.material = mat.clone();
                        }
                        child.userData.isScaleAxis = true;
                        child.userData.axisName = `${config.axis}_axis`;
                        this.scaleGnomonMeshes.push(child);
                    }
                });

                // Store reference to this axis object
                this.scaleGnomonAxisObjects[config.axis] = axisClone;
                axesGroup.add(axisClone);
            }

            this.scaleGnomonAxes = axesGroup;
            this.scaleGnomonGroup.add(axesGroup);
            
            // Load tips for each axis
            const tipLoader = new OBJLoader();
            const tipGeometry = await tipLoader.loadAsync('/models/scale_gnomon_tip.obj');
            
            // Create tips for X, Y, Z axes
            const tipConfigs = [
                { axis: 'x', material: 'axis_x', rotation: [0, 0, -Math.PI / 2] },
                { axis: 'y', material: 'axis_y', rotation: [0, 0, 0] },
                { axis: 'z', material: 'axis_z', rotation: [Math.PI / 2, 0, 0] }
            ];
            
            this.scaleGnomonTips = [];
            for (const config of tipConfigs) {
                const tip = tipGeometry.clone();
                
                // Apply material to tip
                tip.traverse((child) => {
                    if (child instanceof THREE.Mesh) {
                        const mat = materials.materials[config.material];
                        if (mat) {
                            child.material = mat.clone();
                        }
                        child.userData.isScaleAxis = true;
                        child.userData.axisName = `${config.axis}_axis`;
                    }
                });
                
                // Rotate tip to point along the correct axis
                tip.rotation.set(...config.rotation);
                
                // Store tip reference
                this.scaleGnomonTips.push({ mesh: tip, axis: config.axis });
                this.scaleGnomonGroup.add(tip);
                
                // Add tip meshes to raycasting array
                tip.traverse((child) => {
                    if (child instanceof THREE.Mesh) {
                        this.scaleGnomonMeshes.push(child);
                    }
                });
            }
            
            // Add to scene
            this.scene.add(this.scaleGnomonGroup);
            
            console.log('Scale gnomon loaded with', this.scaleGnomonMeshes.length, 'meshes');
        } catch (error) {
            console.error('Failed to load scale gnomon:', error);
        }
    }

    async setupPointedDie() {
        try {
            console.log('Loading pointed die gizmo...');
            
            const objLoader = new OBJLoader();
            const dieModel = await objLoader.loadAsync('/models/pointed_die.obj');
            
            // Get canvas dimensions for render targets
            const canvas = this.renderer.domElement;
            const width = canvas.clientWidth;
            const height = canvas.clientHeight;
            
            // Create render targets for each die (with alpha channel)
            const createRT = () => new THREE.WebGLRenderTarget(width, height, {
                minFilter: THREE.LinearFilter,
                magFilter: THREE.LinearFilter,
                format: THREE.RGBAFormat,
                stencilBuffer: false
            });
            this.referencePointedDieRT = createRT();
            this.transformedPointedDieRT = createRT();
            
            // Create separate scenes for each die (for isolated opaque rendering)
            this.referencePointedDieScene = new THREE.Scene();
            this.referencePointedDieScene.background = null; // Transparent background
            this.transformedPointedDieScene = new THREE.Scene();
            this.transformedPointedDieScene.background = null;
            
            // Add lights to each die scene
            const addLightsToScene = (scene) => {
                scene.add(new THREE.AmbientLight(0xffffff, 0.5));
                const keyLight = new THREE.DirectionalLight(0xffffff, 1.0);
                keyLight.position.set(5, 10, 5);
                scene.add(keyLight);
                const fillLight = new THREE.DirectionalLight(0xffffff, 0.5);
                fillLight.position.set(-5, 5, -5);
                scene.add(fillLight);
            };
            addLightsToScene(this.referencePointedDieScene);
            addLightsToScene(this.transformedPointedDieScene);
            
            // Create reference pointed die (opaque in its own scene)
            this.referencePointedDie = dieModel.clone(true);
            this.referencePointedDie.name = 'reference_pointed_die';
            this.referencePointedDie.traverse((child) => {
                if (child instanceof THREE.Mesh) {
                    // Base color is red, pegs and arrows are lighter red
                    let color = 0xff0000; // Red for cube
                    if (child.name === 'pegs' || child.name === 'arrows') {
                        color = 0xffbbbb; // Light red for pegs and arrows
                    }
                    child.material = new THREE.MeshStandardMaterial({
                        color: color,
                        metalness: 0.1,
                        roughness: 0.5
                    });
                }
            });
            this.referencePointedDie.position.set(0, 0, 0);
            this.referencePointedDieScene.add(this.referencePointedDie);
            
            // Create transformed pointed die (opaque in its own scene)
            this.transformedPointedDie = dieModel.clone(true);
            this.transformedPointedDie.name = 'transformed_pointed_die';
            this.transformedPointedDie.traverse((child) => {
                if (child instanceof THREE.Mesh) {
                    // Base color is green, pegs and arrows are lighter green
                    let color = 0x00ff00; // Green for cube
                    if (child.name === 'pegs' || child.name === 'arrows') {
                        color = 0xbbffbb; // Light green for pegs and arrows
                    }
                    child.material = new THREE.MeshStandardMaterial({
                        color: color,
                        metalness: 0.1,
                        roughness: 0.5
                    });
                }
            });
            this.transformedPointedDie.position.set(2, 1, 1);
            this.transformedPointedDieScene.add(this.transformedPointedDie);
            
            // Create screen-space quad scene for compositing
            this.dieQuadScene = new THREE.Scene();
            this.dieQuadCamera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1);
            
            // Create fullscreen quads to display the render targets
            const quadGeometry = new THREE.PlaneGeometry(2, 2);
            
            // Reference die quad (rendered first, behind)
            const referenceDieQuadMaterial = new THREE.ShaderMaterial({
                uniforms: {
                    tDiffuse: { value: this.referencePointedDieRT.texture },
                    opacity: { value: 0.5 }
                },
                ...DieQuadShader,
                transparent: true,
                depthTest: false,
                depthWrite: false
            });
            this.referenceDieQuad = new THREE.Mesh(quadGeometry, referenceDieQuadMaterial);
            this.referenceDieQuad.renderOrder = 1000;
            this.dieQuadScene.add(this.referenceDieQuad);
            
            // Transformed die quad (rendered second, in front)
            const transformedDieQuadMaterial = new THREE.ShaderMaterial({
                uniforms: {
                    tDiffuse: { value: this.transformedPointedDieRT.texture },
                    opacity: { value: 0.5 }
                },
                ...DieQuadShader,
                transparent: true,
                depthTest: false,
                depthWrite: false
            });
            this.transformedDieQuad = new THREE.Mesh(quadGeometry.clone(), transformedDieQuadMaterial);
            this.transformedDieQuad.renderOrder = 1001;
            this.dieQuadScene.add(this.transformedDieQuad);
            
            console.log('Pointed die gizmo loaded with multi-pass rendering');
        } catch (error) {
            console.error('Failed to load pointed die:', error);
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

    // Convert world position to screen pixels
    worldToScreen(worldPos) {
        const canvas = this.renderer.domElement;
        const rect = canvas.getBoundingClientRect();
        const ndc = worldPos.clone().project(this.camera);
        return new THREE.Vector2(
            (ndc.x + 1) / 2 * rect.width,
            (1 - ndc.y) / 2 * rect.height
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


    // Helper to update gnomon hover state
    updateGnomonHover(raycaster, canvas, meshes, hoverProperty, userDataKey) {
        const intersects = raycaster.intersectObjects(meshes, false);
        
        if (intersects.length > 0) {
            const hitMesh = intersects[0].object;
            const name = hitMesh.userData[userDataKey];
            
            if (this[hoverProperty] !== name) {
                this[hoverProperty] = name;
                canvas.style.cursor = 'pointer';
            }
        } else {
            if (this[hoverProperty]) {
                this[hoverProperty] = null;
            }
        }
    }

    // Helper to start gnomon drag
    startGnomonDrag(event) {
        if (this.controls) {
            this.controls.enabled = false;
        }
        this.renderer.domElement.style.cursor = 'grabbing';
    }

    // Helper to end gnomon drag
    endGnomonDrag() {
        if (this.controls) {
            this.controls.enabled = true;
        }
        // Reset cursor based on current hover state
        const isHovering = this.hoveredTranslationAxis || this.hoveredRotationPlane || this.hoveredScaleAxis;
        this.renderer.domElement.style.cursor = isHovering ? 'pointer' : 'default';
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
            this.startGnomonDrag(event);
        }
        
        // Check if clicking on rotation gnomon axis
        if (this.hoveredRotationPlane) {
            this.draggingRotationPlane = this.hoveredRotationPlane;
            this.rotationDragStartMouse = { x: event.clientX, y: event.clientY };
            this.rotationDragStartQuaternion = this.rotationGnomonGroup.quaternion.clone();
            this.rotationDragStartPosition = this.rotationGnomonGroup.position.clone();
            this.startGnomonDrag(event);
        }
        
        // Check if clicking on scale gnomon axis
        if (this.hoveredScaleAxis) {
            this.draggingScaleAxis = this.hoveredScaleAxis;
            this.scaleDragStartMouse = { x: event.clientX, y: event.clientY };
            this.scaleDragStartScale = this.scaleGnomonScale.clone();
            this.scaleDragStartPosition = this.scaleGnomonGroup.position.clone();
            this.startGnomonDrag(event);
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
        
        // If dragging rotation gnomon axis, handle the drag
        if (this.draggingRotationPlane && this.rotationDragStartMouse) {
            this.isDragging = true;
            this.handleRotationDrag(event);
            return;
        }
        
        // If dragging scale gnomon axis, handle the drag
        if (this.draggingScaleAxis && this.scaleDragStartMouse) {
            this.isDragging = true;
            this.handleScaleDrag(event);
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
        
        // Check for gnomon hover (only when not dragging)
        if (!this.isDragging) {
            const x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
            const y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
            
            const raycaster = new THREE.Raycaster();
            raycaster.setFromCamera(new THREE.Vector2(x, y), this.camera);
            
            // Check all gnomon types
            this.updateGnomonHover(raycaster, canvas, this.translationGnomonMeshes, 'hoveredTranslationAxis', 'axisName');
            this.updateGnomonHover(raycaster, canvas, this.rotationGnomonMeshes, 'hoveredRotationPlane', 'planeName');
            this.updateGnomonHover(raycaster, canvas, this.scaleGnomonMeshes, 'hoveredScaleAxis', 'axisName');
            
            // Set cursor based on any hover state
            if (!this.hoveredTranslationAxis && !this.hoveredRotationPlane && !this.hoveredScaleAxis) {
                canvas.style.cursor = 'default';
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
            this.endGnomonDrag();
        }
        
        // End rotation gnomon drag if active
        if (this.draggingRotationPlane) {
            this.draggingRotationPlane = null;
            this.rotationDragStartMouse = null;
            this.rotationDragStartQuaternion = null;
            this.endGnomonDrag();
        }
        
        // End scale gnomon drag if active
        if (this.draggingScaleAxis) {
            this.draggingScaleAxis = null;
            this.scaleDragStartMouse = null;
            this.scaleDragStartScale = null;
            this.scaleDragStartPosition = null;
            this.endGnomonDrag();
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
        
        // Get local axis direction based on which axis is being dragged
        let localAxisDir = new THREE.Vector3();
        switch (this.draggingTranslationAxis) {
            case 'x_axis': localAxisDir.set(1, 0, 0); break;
            case 'y_axis': localAxisDir.set(0, 1, 0); break;
            case 'z_axis': localAxisDir.set(0, 0, 1); break;
            default: return;
        }

        // Convert local axis to world space
        const worldAxisDir = localAxisDir.applyQuaternion(this.translationGnomonGroup.quaternion);

        // Calculate axis direction in screen space
        const originScreen = this.worldToScreen(this.translationDragStartPosition);
        const axisTipScreen = this.worldToScreen(this.translationDragStartPosition.clone().add(worldAxisDir));
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

    handleRotationDrag(event) {
        if (!this.draggingRotationPlane || !this.rotationDragStartMouse || !this.rotationGnomonGroup) return;
        
        // Determine the rotation axis based on the plane being dragged
        let rotationAxis = new THREE.Vector3();
        let localForwardAxisDir = new THREE.Vector3();
        let localUpAxisDir = new THREE.Vector3();
        switch (this.draggingRotationPlane) {
            case 'xy_plane':
                rotationAxis.set(0, 0, 1);
                localForwardAxisDir.set(1, 0, 0); // x
                localUpAxisDir.set(0, 1, 0); // y
                break; // Z axis
            case 'yz_plane':
                rotationAxis.set(1, 0, 0);
                localForwardAxisDir.set(0, 1, 0); // y
                localUpAxisDir.set(0, 0, 1); // z
                break; // X axis
            case 'zx_plane':
                rotationAxis.set(0, 1, 0);
                localForwardAxisDir.set(0, 0, 1); // z
                localUpAxisDir.set(1, 0, 0); // x
                break; // Y axis
            default: return;
        }



        const worldForwardAxisDir = localForwardAxisDir.applyQuaternion(this.rotationGnomonGroup.quaternion);
        const worldUpAxisDir = localUpAxisDir.applyQuaternion(this.rotationGnomonGroup.quaternion);

        const originScreen = this.worldToScreen(this.rotationDragStartPosition);
        const forwardAxisTipScreen = this.worldToScreen(this.rotationDragStartPosition.clone().add(worldForwardAxisDir));
        const upAxisTipScreen = this.worldToScreen(this.rotationDragStartPosition.clone().add(worldUpAxisDir));

        // Get the screen-space axis vectors (relative to origin)
        const v1 = new THREE.Vector2(
            forwardAxisTipScreen.x - originScreen.x,
            forwardAxisTipScreen.y - originScreen.y
        );
        const v2 = new THREE.Vector2(
            upAxisTipScreen.x - originScreen.x,
            upAxisTipScreen.y - originScreen.y
        );

        // Create 2x2 matrix M = [v1 v2] and compute its inverse
        // M = | v1.x  v2.x |
        //     | v1.y  v2.y |
        // det(M) = v1.x * v2.y - v2.x * v1.y
        const det = v1.x * v2.y - v2.x * v1.y;
        
        // Skip if matrix is singular (axes are parallel in screen space)
        if (Math.abs(det) < 0.0001) return;

        // M^{-1} = (1/det) * |  v2.y  -v2.x |
        //                    | -v1.y   v1.x |
        const invDet = 1.0 / det;

        // Get canvas rect for mouse position calculation
        const canvas = this.renderer.domElement;
        const rect = canvas.getBoundingClientRect();

        // Calculate starting mouse position relative to origin in local coordinates
        const startMouseFromOrigin = new THREE.Vector2(
            this.rotationDragStartMouse.x - rect.left - originScreen.x,
            this.rotationDragStartMouse.y - rect.top - originScreen.y
        );
        const startLocal = new THREE.Vector2(
            invDet * (v2.y * startMouseFromOrigin.x - v2.x * startMouseFromOrigin.y),
            invDet * (-v1.y * startMouseFromOrigin.x + v1.x * startMouseFromOrigin.y)
        );

        // Calculate current mouse position relative to origin in local coordinates
        const currentMouseFromOrigin = new THREE.Vector2(
            event.clientX - rect.left - originScreen.x,
            event.clientY - rect.top - originScreen.y
        );
        const currentLocal = new THREE.Vector2(
            invDet * (v2.y * currentMouseFromOrigin.x - v2.x * currentMouseFromOrigin.y),
            invDet * (-v1.y * currentMouseFromOrigin.x + v1.x * currentMouseFromOrigin.y)
        );

        // Calculate angles in the local 2D plane
        const startAngle = Math.atan2(startLocal.y, startLocal.x);
        const currentAngle = Math.atan2(currentLocal.y, currentLocal.x);
        const deltaAngle = currentAngle - startAngle;

        // Convert local rotation axis to world space
        const worldRotationAxis = rotationAxis.clone().applyQuaternion(this.rotationDragStartQuaternion);

        // Create rotation quaternion around the world axis
        const deltaQuat = new THREE.Quaternion().setFromAxisAngle(worldRotationAxis, deltaAngle);

        // Apply rotation: new = delta * start
        this.rotationGnomonGroup.quaternion.copy(deltaQuat.multiply(this.rotationDragStartQuaternion));
    }

    // Update scale gnomon tip positions based on current scale
    updateScaleGnomonTips() {
        if (!this.scaleGnomonAxes || !this.scaleGnomonTips.length) return;
        
        const scale = this.scaleGnomonScale;
        
        for (const tipData of this.scaleGnomonTips) {
            const { mesh, axis } = tipData;
            
            // Position tip at the end of each scaled axis
            switch (axis) {
                case 'x':
                    mesh.position.set(1.0 * scale.x, 0, 0);
                    break;
                case 'y':
                    mesh.position.set(0, 1.0 * scale.y, 0);
                    break;
                case 'z':
                    mesh.position.set(0, 0, 1.0 * scale.z);
                    break;
            }
        }
    }

    handleScaleDrag(event) {
        if (!this.draggingScaleAxis || !this.scaleDragStartMouse || !this.scaleGnomonGroup) return;

        const canvas = this.renderer.domElement;
        const rect = canvas.getBoundingClientRect();

        // Determine which axis is being scaled
        let localAxisDir = new THREE.Vector3();
        switch (this.draggingScaleAxis) {
            case 'x_axis': localAxisDir.set(1, 0, 0); break;
            case 'y_axis': localAxisDir.set(0, 1, 0); break;
            case 'z_axis': localAxisDir.set(0, 0, 1); break;
            default: return;
        }

        // Convert local axis to world space using SCALE gnomon's quaternion
        const worldAxisDir = localAxisDir.applyQuaternion(this.scaleGnomonGroup.quaternion);

        // Calculate axis direction in screen space
        const originScreen = this.worldToScreen(this.scaleDragStartPosition);
        const axisTipScreen = this.worldToScreen(this.scaleDragStartPosition.clone().add(worldAxisDir));
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
        let newScale = this.scaleDragStartScale.clone();

        switch (this.draggingScaleAxis) {
            case 'x_axis': newScale.x = projectionScalar; break;
            case 'y_axis': newScale.y = projectionScalar; break;
            case 'z_axis': newScale.z = projectionScalar; break;
            default: return;
        }

        // Store initial projection on first frame
        if (this.scaleDragDistance === null) {
            this.scaleDragDistance = projectionScalar;
        }

        // Update scale
        this.scaleGnomonScale.copy(newScale);

        // Apply visual scale to cylinders (only stretch along length, not diameter)
        // Each cylinder is oriented along Z in its local space
        if (this.scaleGnomonAxisObjects.x) {
            this.scaleGnomonAxisObjects.x.scale.set(1, 1, newScale.x);
        }
        if (this.scaleGnomonAxisObjects.y) {
            this.scaleGnomonAxisObjects.y.scale.set(1, 1, newScale.y);
        }
        if (this.scaleGnomonAxisObjects.z) {
            this.scaleGnomonAxisObjects.z.scale.set(1, 1, newScale.z);
        }
        
        // Update tip positions
        this.updateScaleGnomonTips();
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

    animate() {
        requestAnimationFrame(() => this.animate());
        
        // Sync Rust scene with Three.js scene
        this.syncScene();
        
        this.controls.update();
        
        // Multi-pass rendering for pointed dice
        if (this.referencePointedDieScene && this.transformedPointedDieScene) {
            // Render reference die to its render target (opaque)
            this.renderer.setRenderTarget(this.referencePointedDieRT);
            this.renderer.setClearColor(0x000000, 0); // Clear to transparent
            this.renderer.clear();
            this.renderer.render(this.referencePointedDieScene, this.camera);
            
            // Render transformed die to its render target (opaque)
            this.renderer.setRenderTarget(this.transformedPointedDieRT);
            this.renderer.setClearColor(0x000000, 0); // Clear to transparent
            this.renderer.clear();
            this.renderer.render(this.transformedPointedDieScene, this.camera);
            
            // Reset render target to screen
            this.renderer.setRenderTarget(null);
        }
        
        // Render main scene with composer (post-processing)
        this.composer.render();
        
        // Composite the translucent dice on top
        if (this.dieQuadScene && this.dieQuadCamera) {
            this.renderer.autoClear = false;
            this.renderer.render(this.dieQuadScene, this.dieQuadCamera);
            this.renderer.autoClear = true;
        }
    }
}

// Initialize the app
const app = new DeltaBrush();
app.init().catch(console.error);
