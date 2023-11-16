use std::f32::consts::PI;
use std::fs::File;
use std::io::{self, Write};

use nalgebra::Vector3;

#[derive(Debug, Clone, Copy)]
struct Material {
    diffuse_color: Vector3<f32>,
}

impl Material {
    fn new(diffuse_color: Vector3<f32>) -> Self {
        Self { diffuse_color }
    }

    fn diffuse(&self) -> Vector3<f32> {
        self.diffuse_color.clone()
    }
}

struct Sphere {
    center: Vector3<f32>,
    radius: f32,
    material: Material,
}

impl Sphere {
    fn new(center: Vector3<f32>, radius: f32, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }

    fn ray_intersect(
        &self,
        origin: Vector3<f32>,
        direction: Vector3<f32>, /* _distance: f32 */
    ) -> Option<Material> {
        // Calculate the direction vector of the line segment
        let dir_normalized = direction.normalize();
        // Calculate the vector from the line start to the sphere center
        let start_to_center = self.center - origin;
        // Calculate the projection of start_to_center onto the line direction
        let projection = start_to_center.dot(&dir_normalized);
        // Calculate the closest point on the line to the sphere center
        let closest_point = origin + dir_normalized * projection;
        // Calculate the distance between the closest point and the sphere center
        let distance = (closest_point - self.center).norm();
        // Check if the closest point is within the sphere
        if distance <= self.radius {
            Some(self.material)
        } else {
            None
        }
    }
}

fn cast_ray(
    origin: Vector3<f32>,
    direction: Vector3<f32>,
    spheres: &Vec<Sphere>,
) -> Option<Vector3<f32>> {
    // Maybe get the closest sphere first? How do we do that?
    //
    // A sphere with a small radius could be occluded by a sphere that is farther away
    // but has a very large radius.
    //
    // If we can get the distance between the origin and the center of the sphere plus
    // the radius, then we can compare each sphere in this manner and determine the
    // closest one, using that one to determine our ray intersection.
    //
    // But this won't work for other, more complicated shapes.
    let sphere: &Sphere = &spheres[0];
    match spheres
        .iter()
        .find(|s| s.ray_intersect(origin, direction).is_some())
    {
        Some(s) => Some(s.material.diffuse()),
        None => None,
    }
}

fn write_ppm_image(filename: &str, width: u32, height: u32, pixels: &[u8]) -> io::Result<()> {
    let mut file = File::create(filename)?;

    // Write the PPM header
    writeln!(file, "P6 {} {} 255", width, height)?;

    // Write the pixel data
    file.write_all(pixels)?;

    Ok(())
}

fn vec_to_rgb(v: Vector3<f32>) -> (u8, u8, u8) {
    // 255 * (max of 0.0 or (min of 1.0 or v[d]))
    // for each channel
    (
        ((255.0 * v.x.clamp(0.0, 1.0)) as u8),
        ((255.0 * v.y.clamp(0.0, 1.0)) as u8),
        ((255.0 * v.z.clamp(0.0, 1.0)) as u8),
    )
}

fn render(spheres: Vec<Sphere>) -> io::Result<()> {
    let width = 256 * 2;
    let height = 256 * 2;
    let fov = PI / 3.0;
    let filename = "output.ppm";

    let mut pixels = Vec::new();
    for j in 0..height {
        for i in 0..width {
            let x = (2.0 * (i as f32 + 0.5) / width as f32 - 1.0)
                * (fov / 2.0).tan()
                * (width as f32 / height as f32);
            let y = -(2.0 * (j as f32 + 0.5) / height as f32 - 1.0) * (fov / 2.0).tan();
            let dir = Vector3::new(x, y, -1.0).normalize();
            let (r, g, b) = match cast_ray(Vector3::zeros(), dir, &spheres) {
                Some(v) => vec_to_rgb(v),
                _ => vec_to_rgb(Vector3::new(
                    j as f32 / height as f32,
                    i as f32 / width as f32,
                    (i + j) as f32 / (height + width) as f32,
                )),
            };
            pixels.push(r);
            pixels.push(g);
            pixels.push(b);
        }
    }

    write_ppm_image(filename, width, height, &pixels)
}

fn main() -> io::Result<()> {
    let chartreuse = Material::new(Vector3::new(0.5, 0.8, 0.3));
    let red = Material::new(Vector3::new(1.0, 0.5, 0.5));
    let spheres = vec![
        Sphere::new(Vector3::new(-3.0, 0.0, -16.0), 2.0, chartreuse),
        Sphere::new(Vector3::new(2.0, 1.0, -16.0), 5.0, red),
    ];
    render(spheres)?;
    Ok(())
}
