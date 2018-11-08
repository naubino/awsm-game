#![allow(unused_imports)]
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use web_sys::{EventTarget, KeyboardEvent};

use serde_derive::{Serialize, Deserialize};

use nalgebra::{Vector2, zero};
use ncollide2d::shape::{Cuboid};
use ncollide2d::world::CollisionObjectHandle;
use nphysics2d::object::{BodyHandle, Material};
use nphysics2d::volumetric::Volumetric;

use crate::{debug, SceneConfig};

type World = nphysics2d::world::World<f64>;
type Isometry2 = nalgebra::Isometry2<f64>;
type ShapeHandle = ncollide2d::shape::ShapeHandle<f64>;

macro_rules! debug { ($($arg:tt)*) => (debug(&format!($($arg)*));) }

pub struct Brick {
    pub shape: ShapeHandle,
    pub body: BodyHandle,
    pub collision_object: CollisionObjectHandle,
    pub uid: usize,
    pub ttl: Option<f64>,
    pub fill_style: String,
}

impl Brick {
    pub fn new(world: &mut World, transform: Isometry2, radx: f64, rady: f64, margin: f64, fill_style: &str) -> Brick {
        let shape = ShapeHandle::new(Cuboid::new(Vector2::new(radx, rady)));
        let body = world.add_rigid_body(transform, shape.inertia(0.1), shape.center_of_mass());
        let collision_object = world.add_collider(
            margin,
            shape.clone(),
            body,
            Isometry2::identity(),
            Material::new(0.0, 1.0)
        );
        let uid = collision_object.uid();
        let ttl: Option<f64> = None;
        Brick { shape, body, collision_object, uid, ttl, fill_style: fill_style.into() }
    }

    pub fn from_vector(world: &mut World, vector: Vector2<f64>, radx: f64, rady: f64, margin: f64, fill_style: &str) -> Self {
        let pos = Isometry2::new(vector, 0.0);
        Brick::new(world, pos, radx, rady, margin, fill_style)
    }
}

pub fn make_ground(world: &mut World, cfg: &SceneConfig) -> CollisionObjectHandle {
    let margin = cfg.margin.unwrap_or(0.);
    let radius_x = cfg.ground_radx.unwrap_or(1.);
    let radius_y = cfg.ground_rady.unwrap_or(1.);
    let radius = Vector2::new(radius_x, radius_y);
    let cuboid = Cuboid::new(radius);
    let shape = ShapeHandle::new(cuboid);
    let x = cfg.ground_x.unwrap_or(0.);
    let y = cfg.ground_y.unwrap_or(0.);
    let pos = Isometry2::new(Vector2::new(x, y), zero());
    debug!("make_ground {:?}", (margin, radius_x, radius_y, radius, pos));

    world.add_collider(
        margin,
        shape,
        BodyHandle::ground(),
        pos,
        Material::default(),
    )
}

pub fn make_building(world: &mut World, x_pos: f64, width: usize, height: usize, fill_style: &str, cfg: &SceneConfig) -> Vec<Brick> {

    let margin = cfg.margin.unwrap_or(0.);
    let radx = cfg.box_radx.unwrap_or(1.);
    let rady = cfg.box_rady.unwrap_or(1.);
    let ground_x = cfg.ground_x.unwrap_or(0.);
    let ground_y = cfg.ground_y.unwrap_or(0.);
    let ground_rady = cfg.ground_rady.unwrap_or(0.);

    let w = radx + margin;
    let h = rady + margin;

    let mut bricks = Vec::with_capacity(height * (width + 1) + 4);

    for yi in 0..height {
        for xi in 0..width {
            let x = xi as f64;
            let y = yi as f64;

            let x_offset = if yi % 2 == 0 { radx } else { 0f64 };

            if yi % 2 == 0 && xi == 0 {
                let corner_pos = Vector2::new(
                    x_pos + x * 1.0 * w,
                    ground_y - ground_rady - h * (1. + margin + 2. * (y + margin)));
                bricks.push(Brick::from_vector(world, corner_pos, radx * 0.5, rady, margin, fill_style));
            }
            if yi % 2 == 1 && xi == width-1 {
                let corner_pos = Vector2::new(
                    x_pos + x * 1.0 * w + w * width as f64,
                    ground_y - ground_rady - h * (1. + margin + 2. * (y + margin)));
                bricks.push(Brick::from_vector(world, corner_pos, radx * 0.5, rady, margin, fill_style));
            }

            let pos = Vector2::new(
                x_pos + x * 2.0 * w + x_offset,
                ground_y - ground_rady - h * (2. * (y))
                );


            bricks.push(Brick::from_vector(world, pos, radx, rady, margin, fill_style));
        }
    }
    bricks.push(
        Brick::from_vector(
            world,
            Vector2::new(
                x_pos + width as f64 * radx * 0.5 + radx * 2.0,
                ground_y - ground_rady - h * (2. * (height as f64))
            ),
            radx * width as f64 + radx * 0.5,
            rady,
            margin,
            fill_style
        )
    );
    bricks
}
