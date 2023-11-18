use std::f32::consts::PI;
use std::fs::File;
use std::io::{self, Write};

use nalgebra::{Matrix3x1, SimdValue, Vector3};

#[derive(Debug, Clone, Copy)]
struct Light {
    position: Vector3<f32>,
    intensity: f32,
}

impl Light {
    fn new(position: Vector3<f32>, intensity: f32) -> Self {
        Self {
            position,
            intensity,
        }
    }

    fn pos(&self) -> Vector3<f32> {
        self.position.clone()
    }

    fn diffuse_for_intersection(&self, intersection: &Intersection) -> f32 {
        let direction = (self.pos() - intersection.point).normalize();
        self.intensity * f32::max(0.0, (direction * intersection.distance).norm())
    }
}

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
    ) -> Option<Intersection> {
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
            Some(Intersection::new(closest_point, distance, self.material))
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct Intersection {
    point: Vector3<f32>,
    distance: f32,
    material: Material,
}

impl Intersection {
    fn new(point: Vector3<f32>, distance: f32, material: Material) -> Self {
        Self {
            point,
            distance,
            material,
        }
    }
}

fn scene_intersect(
    origin: Vector3<f32>,
    direction: Vector3<f32>,
    spheres: &Vec<Sphere>,
) -> Option<Intersection> {
    // Find the nearest intersection if it exists and return it
    let nearest = spheres
        .iter()
        .fold(None, |nearest: Option<Intersection>, sphere| {
            match sphere.ray_intersect(origin, direction) {
                Some(intersection) => match nearest {
                    Some(nearest) if intersection.distance < nearest.distance => Some(intersection),
                    _ => Some(intersection),
                },
                None => nearest,
            }
        });

    match nearest {
        Some(intersection) if intersection.distance < 1000.0 => Some(intersection),
        _ => None,
    }
}

fn cast_ray(
    origin: Vector3<f32>,
    direction: Vector3<f32>,
    spheres: &Vec<Sphere>,
    lights: &Vec<Light>,
) -> Option<Vector3<f32>> {
    let intersection = match scene_intersect(origin, direction, spheres) {
        Some(intersection) => intersection,
        _ => return None,
    };

    let diffuse_intensity: f32 = lights.iter().fold(0.0, |acc: f32, light: &Light| {
        acc + light.diffuse_for_intersection(&intersection)
    });

    Some(intersection.material.diffuse() as Vector3<f32> + Vector3::repeat(diffuse_intensity))
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

fn render(spheres: &Vec<Sphere>, lights: &Vec<Light>) -> io::Result<()> {
    let width = 256;
    let height = 256;
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
            let (r, g, b) = match cast_ray(Vector3::zeros(), dir, &spheres, &lights) {
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
        Sphere::new(Vector3::new(2.0, 1.0, -16.0), 5.0, red),
        Sphere::new(Vector3::new(2.0, 3.0, -11.0), 1.0, chartreuse),
        Sphere::new(Vector3::new(-3.0, 0.0, -16.0), 2.0, chartreuse),
    ];
    let lights = vec![Light::new(Vector3::new(-20.0, 20.0, 20.0), 0.2)];
    render(&spheres, &lights)?;
    Ok(())
}
