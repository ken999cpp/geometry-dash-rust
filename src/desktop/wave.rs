use bevy::{asset::RenderAssetUsages, mesh::PrimitiveTopology, prelude::*};

/// One joined ribbon: adjacent segments share their two corner vertices.
/// There are no round caps, circles or interpolated curves at direction changes.
pub(super) fn ribbon(points: &[Vec2], width: f32) -> Mesh {
    let mut positions = Vec::with_capacity(points.len().saturating_sub(1) * 6);
    let mut edges = Vec::with_capacity(points.len());
    for (i, &p) in points.iter().enumerate() {
        let before = if i > 0 {
            p - points[i - 1]
        } else {
            points.get(1).copied().unwrap_or(p) - p
        };
        let after = points.get(i + 1).map_or(before, |next| *next - p);
        let a = before.normalize_or_zero().perp();
        let b = after.normalize_or_zero().perp();
        let bisector = (a + b).normalize_or_zero();
        let offset = bisector * (width * 0.5 / bisector.dot(b).max(0.25));
        edges.push([p + offset, p - offset]);
    }
    for pair in edges.windows(2) {
        for p in [
            pair[0][0], pair[0][1], pair[1][0], pair[1][0], pair[0][1], pair[1][1],
        ] {
            positions.push([p.x, p.y, 0.]);
        }
    }
    let count = positions.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0., 0., 1.]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0., 0.]; count])
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    #[test]
    fn sharp_turns_share_the_same_seam_and_keep_constant_width() {
        let mesh = ribbon(&[Vec2::ZERO, Vec2::new(20., 20.), Vec2::new(40., 0.)], 6.);
        let Some(VertexAttributeValues::Float32x3(p)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("positions")
        };
        assert_eq!(p.len(), 12);
        assert_eq!(p[2], p[6]);
        assert_eq!(p[5], p[7]);
        let corner = Vec2::new(p[2][0], p[2][1]);
        assert!(
            ((corner - Vec2::new(20., 20.))
                .dot(Vec2::new(-1., 1.).normalize())
                .abs()
                - 3.)
                .abs()
                < 1e-5
        );
        assert!(p.iter().flatten().all(|v| v.is_finite()));
    }
}
