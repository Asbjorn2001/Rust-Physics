use graphics::{color, line, math::Matrix2d, Context};
use opengl_graphics::{GlGraphics, Texture};
use graphics::triangulation::*;
use piston_window::Graphics;
use crate::Vector2f;

use super::{circle::Circle, shape_type::ShapeType, polygon::Polygon};

const TILE_WIDTH: u32 = 64;
const TILE_HEIGHT: u32 = 64;

#[derive(Clone)]
struct Mesh {
    verts: Vec<[f32; 2]>,
    uvs: Vec<[f32; 2]>,
    tile_verts: Vec<[f64; 2]>
}

#[derive(Clone)]
pub struct TiledMesh {
    tiles: Vec<Mesh>,
}

impl TiledMesh {
    pub fn draw(&self, transform: Matrix2d, color: [f32; 4], texture: &Texture, c: Context, gl: &mut GlGraphics) {
        for mesh in self.tiles.as_slice() {
            let v: Vec<[f32; 2]> = mesh.verts.iter().map(|v| [tx(transform, v[0] as f64, v[1] as f64), ty(transform, v[0] as f64, v[1] as f64)]).collect();
            gl.tri_list_uv(&c.draw_state, &color, texture, |f| {
                f(
                    &v,
                    &mesh.uvs,
                )
            });
        }
    }

    pub fn draw_tile_outline(&self, transform: Matrix2d, gl: &mut GlGraphics) {
        for mesh in self.tiles.as_slice() {
            for i in 0..mesh.tile_verts.len() {
                let a = mesh.tile_verts[i];
                let b = mesh.tile_verts[(i + 1) % mesh.tile_verts.len()];
                let l = [a[0], a[1], b[0], b[1]];
                line(color::RED, 1.0, l, transform, gl);
            }
        }
    }
}

impl From<&ShapeType> for TiledMesh {
    fn from(value: &ShapeType) -> Self {
        match value {
            ShapeType::Circle(circle) => Self::from(circle),
            ShapeType::Polygon(poly) => Self::from(poly),
        }
    }
}

impl From<&Circle> for TiledMesh {
    fn from(circle: &Circle) -> Self {
        let r = circle.radius as i32;
        let verts = Polygon::new_regular_polygon(
            circle.radius as u32 * 3, circle.radius, Vector2f::new(0.0, 0.0), color::WHITE).local_vertices;

        Self { 
            tiles: generate_tiles(-r, r, -r, r, verts),
        }
    }
}

impl From<&Polygon> for TiledMesh {
    fn from(poly: &Polygon) -> Self {
        let verts = poly.local_vertices.clone();

        let min_x = verts.iter().map(|v| v.x as i32).min().unwrap();
        let max_x = verts.iter().map(|v| v.x as i32).max().unwrap();
        let min_y = verts.iter().map(|v| v.y as i32).min().unwrap();
        let max_y = verts.iter().map(|v| v.y as i32).max().unwrap();
        
        Self { 
            tiles: generate_tiles(min_x, max_x, min_y, max_y, verts),
        }
    }
}

fn generate_tiles(min_x: i32, max_x: i32,  min_y: i32, max_y: i32, clip_verts: Vec<Vector2f<f64>>) -> Vec<Mesh> {
    let width = (max_x - min_x) as f64;
    let height = (max_y - min_y) as f64;

    let tile_width = TILE_WIDTH as f64;
    let tile_height = TILE_HEIGHT as f64;

    let ratio_x = width / tile_width;
    let ratio_y = height / tile_height;

    let tiles_x = ratio_x.ceil() as usize;
    let tiles_y = ratio_y.ceil() as usize;

    let mut tiles = vec![];
    for i in 0..tiles_x {
        for j in 0..tiles_y {
            let tile_min_x = min_x as f64 + i as f64 * tile_width;
            let tile_max_x = tile_min_x + tile_width;
            let tile_min_y = min_y as f64 + j as f64 * tile_height;
            let tile_max_y = tile_min_y + tile_height;

            let mut tile_box = vec![
                Vector2f::new(tile_min_x, tile_min_y),
                Vector2f::new(tile_max_x, tile_min_y),
                Vector2f::new(tile_max_x, tile_max_y),
                Vector2f::new(tile_min_x, tile_max_y)
            ];

            suth_hodg_clip(&mut tile_box, &clip_verts);
            
            let mut tiles_uv = vec![];
            let mut tiles_xy = vec![];
            for v in tile_box {
                let uv_x = (v.x - tile_min_x) / tile_width; 
                let uv_y = 1.0 - (v.y - tile_min_y) / tile_height;
                tiles_uv.push([uv_x, uv_y]);
                tiles_xy.push([v.x, v.y]);
            }
            tiles.push((tiles_xy, tiles_uv));
        }
    }

    let identity_matrix = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
    let mut final_tiles = vec![];
    for (tiles_xy, tiles_uv) in tiles {
        let mut tri_verts = vec![];
        stream_polygon_tri_list(identity_matrix, tiles_xy.clone().into_iter(), |f| {
            for v in f {
                tri_verts.push(*v);
            }
        });

        let mut uvs = vec![];
        stream_polygon_tri_list(identity_matrix, tiles_uv.into_iter(), |f| {
            for uv in f {
                uvs.push(*uv);
            }
        });
        final_tiles.push(Mesh { verts: tri_verts, uvs: uvs, tile_verts: tiles_xy });
    }

    final_tiles
}


// Returns the point of intersection between two lines (p1, p2) and (p3, p4)
fn intersection_line_line(p1: Vector2f<f64>, p2: Vector2f<f64>, p3: Vector2f<f64>, p4: Vector2f<f64>) -> Option<Vector2f<f64>> {
    let denom = (p1.x - p2.x) * (p3.y - p4.y) - (p1.y - p2.y) * (p3.x - p4.x);

    if denom.abs() < 1e-10 {
        return None; // Lines are parallel or coincident
    }

    let x = ((p1.x * p2.y - p1.y * p2.x) * (p3.x - p4.x) - (p1.x - p2.x) * (p3.x * p4.y - p3.y * p4.x)) / denom;
    let y = ((p1.x * p2.y - p1.y * p2.x) * (p3.y - p4.y) - (p1.y - p2.y) * (p3.x * p4.y - p3.y * p4.x)) / denom;

    Some(Vector2f::new(x, y))
}

fn is_inside(poly: Vector2f<f64>, a: Vector2f<f64>, b: Vector2f<f64>) -> bool {
    let ab = b - a;
    let ap = poly - a;
    ap.cross(ab) <= 0.0
}

fn clip(poly: &mut Vec<Vector2f<f64>>, a: Vector2f<f64>, b: Vector2f<f64>) {
    let mut new_poly = vec![];

    for i in 0..poly.len() {
        let current = poly[i];
        let next = poly[(i + 1) % poly.len()];

        let current_inside = is_inside(current, a, b);
        let next_inside = is_inside(next, a, b);

        if current_inside && next_inside {
            new_poly.push(next);
        } else if current_inside && !next_inside {
            if let Some(intersect) = intersection_line_line(current, next, a, b) {
                new_poly.push(intersect);
            }
        } else if !current_inside && next_inside {
            if let Some(intersect) = intersection_line_line(current, next, a, b) {
                new_poly.push(intersect);
            }
            new_poly.push(next);
        }
        // else: both outside, do nothing
    }

    poly.clear();
    poly.extend(new_poly);
}

// Source: https://www.geeksforgeeks.org/polygon-clipping-sutherland-hodgman-algorithm/
fn suth_hodg_clip(subject_polygon: &mut Vec<Vector2f<f64>>, clip_polygon: &Vec<Vector2f<f64>>) {
    for i in 0..clip_polygon.len() {
        let a = clip_polygon[i];
        let b = clip_polygon[(i + 1) % clip_polygon.len()];
        clip(subject_polygon, a, b);
    }
}
