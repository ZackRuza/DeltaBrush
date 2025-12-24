use crate::Mesh;

use ahash::AHashMap;
use std::io::Cursor;

/// Parse OBJ text into DeltaBrush's flat triangle `Mesh`.
///
/// Behavior:
/// - Forces triangulation.
/// - Requests single-index output.
/// - Merges all models/shapes into one `Mesh`.
/// - Ignores UVs/normals/materials.
pub fn parse_obj_to_mesh(obj_text: &str) -> Result<Mesh, String> {
	let mut reader = Cursor::new(obj_text.as_bytes());

	let load_options = tobj::LoadOptions {
		triangulate: true,
		single_index: true,
		..Default::default()
	};

	let (models, _materials) = tobj::load_obj_buf(
		&mut reader,
		&load_options,
		// Ignore .mtl loading for now.
		|_| Ok((Vec::new(), AHashMap::new())),
	)
	.map_err(|e| format!("OBJ parse failed: {e}"))?;

	let mut out = Mesh::new();

	for model in models {
		let positions = &model.mesh.positions;
		if positions.len() % 3 != 0 {
			return Err("OBJ positions are not a multiple of 3".to_string());
		}

		let base_vertex = (out.vertex_coords.len() / 3) as u32;
		out.vertex_coords.extend_from_slice(positions);

		let indices = &model.mesh.indices;
		if indices.len() % 3 != 0 {
			return Err("OBJ indices are not a multiple of 3 (triangulation failed?)".to_string());
		}

		out.face_indices
			.extend(indices.iter().map(|i| i + base_vertex));
	}

	Ok(out)
}
