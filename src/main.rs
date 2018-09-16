extern crate nalgebra as na;
extern crate ncollide2d;
extern crate quicksilver;

use ncollide2d::{
    shape::{Cuboid, Shape, ShapeHandle},
    world::{CollisionGroups, CollisionObjectHandle, CollisionWorld, GeometricQueryType}
};
use quicksilver::{
    *,
    geom::*,
    graphics::*,
    input::*,
    lifecycle::*,
    sound::*,
};

mod unique_store;
use unique_store::{Key, KeyAllocator, UniqueStore};

const PIXELS_PER_UNIT: f32 = 32.0;
const PLAYER_SIZE: Vector = Vector {
    x: 1.0,
    y: 1.0,
};

#[derive(Copy, Clone)]
enum CollisionProp {
    Terrain,
    Entity(Key),
}

struct Store {
    world: CollisionWorld<f32, CollisionProp>,
    key_alloc: KeyAllocator,
    bounds: UniqueStore<CollisionObjectHandle>,
    player: Key,
}

impl Store {
    fn new() -> Store {
        let mut state = Store {
            world: CollisionWorld::new(0.02),
            key_alloc: KeyAllocator::new(),
            bounds: UniqueStore::new(),
            player: Key::null(),
        };
        state.player = state.add((0, 0), 0, Cuboid::new((PLAYER_SIZE / 2).into_vector()), true);
        state
    }

    fn add(&mut self, pos: impl Into<Vector>, angle: impl Scalar, bounds: impl Shape<f32>, solid: bool) -> Key {
        let key = self.key_alloc.alloc();
        let pos: Vector = pos.into();
        let bounds = ShapeHandle::new(bounds);
        let query_type = match solid {
            true => GeometricQueryType::Contacts(0.02, 0.02),
            false => GeometricQueryType::Proximity(0.02),
        };
        let isometry = na::Isometry2::new(pos.into_vector(), angle.float());
        self.bounds.insert(key, self.world.add(isometry, bounds, CollisionGroups::new(), query_type, CollisionProp::Entity(key)));
        key
    }
}

struct Game {
    store: Store
}

impl State for Game {
    fn new() -> Result<Game> {
        Ok(Game {
            store: Store::new()
        })
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        use Background::*;
        window.clear(Color::BLACK)?;
        window.draw(&Rectangle::new((50, 50), (50, 50)), Col(Color::WHITE));
        Ok(())
    }
}

fn main() {
    run::<Game>("Rebound", Vector::new(960, 540), Settings::default());
}
