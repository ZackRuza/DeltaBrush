use serde::{Deserialize, Serialize, Serializer, Deserializer};
use crate::{Material, Mesh, MeshEditor, Point3, Transform, Transformable, algorithms::moller_trumbore_intersection_exterior_algebra, geometry::{HitResponse, Ray3}, mesh};


#[derive(Clone)]
pub enum Model {
    StaticMesh(Mesh),
    EditableMesh(MeshEditor),
}

#[derive(Clone)]
pub struct SceneObject {
    pub id: usize,
    pub model: Model,
    pub transform: Transform,
    pub material: Material,
}

/// World hit response holds the hit response in world coordinates, as well as the
/// distance and object ID
#[derive(Clone)]
pub struct WorldHitResponse {
    pub hit_response: HitResponse,
    pub distance: f32,
    pub object_id: usize,
}

impl SceneObject {
    /// Get the current renderable mesh
    pub fn get_mesh(&self) -> &Mesh {
        match &self.model {
            Model::StaticMesh(mesh) => mesh,
            Model::EditableMesh(editor) => editor.get_mesh(),
        }
    }

    pub fn raycast_closest_hit(&self, ray: Ray3) -> Option<WorldHitResponse> {
        let mesh = self.get_mesh();
        let transform = &self.transform;
        let transformed_ray = ray.inverse_transform(transform);
        let mut closest: Option<WorldHitResponse> = None;

        // Go through each triangle and perform ray intersection
        let vert_coords = &mesh.vertex_coords;
        let mut chunks = mesh.face_indices.chunks_exact(3);
        for tri in &mut chunks {
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;

            let p = |i: usize| Point3::new(vert_coords[3 * i], vert_coords[3 * i + 1], vert_coords[3 * i + 2]);
            
            if let Some(this_hit)
                = moller_trumbore_intersection_exterior_algebra(transformed_ray, p(i0), p(i1), p(i2)) {
                
                // The hit response was in local coordinates. Transform to world coordinates.
                let world_hit = this_hit.transform(transform);

                let this_world_distance = world_hit.hit_direction.length();
                let should_update = match &closest {
                    None => true,
                    Some(existing) =>
                        this_world_distance < existing.distance,
                };

                if should_update {
                    closest = Some(WorldHitResponse {
                        hit_response: world_hit,
                        distance: this_world_distance,
                        object_id: self.id,
                    });
                }
            }
        }

        if !chunks.remainder().is_empty() {
            crate::console_log!("Mesh indices not a multiple of 3. Trailing mesh indices ignored.");
        }

        closest
    }
}

// Custom serialization - JavaScript always sees the current renderable mesh
impl Serialize for SceneObject {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("SceneObject", 4)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("mesh", self.get_mesh())?;
        state.serialize_field("transform", &self.transform)?;
        state.serialize_field("material", &self.material)?;
        state.end()
    }
}

// Deserialization always creates static mesh objects
impl<'de> Deserialize<'de> for SceneObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field { Id, Mesh, Transform, Material }

        struct SceneObjectVisitor;

        impl<'de> Visitor<'de> for SceneObjectVisitor {
            type Value = SceneObject;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct SceneObject")
            }

            fn visit_map<V>(self, mut map: V) -> Result<SceneObject, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id = None;
                let mut mesh = None;
                let mut transform = None;
                let mut material = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Id => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        Field::Mesh => {
                            if mesh.is_some() {
                                return Err(de::Error::duplicate_field("mesh"));
                            }
                            mesh = Some(map.next_value()?);
                        }
                        Field::Transform => {
                            if transform.is_some() {
                                return Err(de::Error::duplicate_field("transform"));
                            }
                            transform = Some(map.next_value()?);
                        }
                        Field::Material => {
                            if material.is_some() {
                                return Err(de::Error::duplicate_field("material"));
                            }
                            material = Some(map.next_value()?);
                        }
                    }
                }

                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let mesh = mesh.ok_or_else(|| de::Error::missing_field("mesh"))?;
                let transform = transform.ok_or_else(|| de::Error::missing_field("transform"))?;
                let material = material.ok_or_else(|| de::Error::missing_field("material"))?;

                Ok(SceneObject {
                    id,
                    model: Model::StaticMesh(mesh),
                    transform,
                    material,
                })
            }
        }

        const FIELDS: &[&str] = &["id", "mesh", "transform", "material"];
        deserializer.deserialize_struct("SceneObject", FIELDS, SceneObjectVisitor)
    }
}
