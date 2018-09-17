extern crate nalgebra as na;
extern crate ncollide2d;
extern crate quicksilver;

use ncollide2d::{
    event::ContactEvent,
    shape::{Cuboid, Shape, ShapeHandle},
    world::{CollisionGroups, CollisionObjectHandle, CollisionWorld, GeometricQueryType}
};
use quicksilver::{
    *,
    geom::{*, Shape as ShapeTrait},
    graphics::*,
    input::*,
    lifecycle::*,
    sound::*,
};

mod unique_store;
use unique_store::{Key, KeyAllocator, UniqueStore, from};

const PIXELS_PER_UNIT: f32 = 100.0;
const PLAYER_SIZE: Vector = Vector {
    x: 0.16,
    y: 0.16,
};

#[derive(Copy, Clone)]
enum CollisionProp {
    Terrain,
    Entity(Key),
}

#[derive(Copy, Clone)]
enum EntityType {
    Player
}

use EntityType::*;

struct Store {
    world: CollisionWorld<f32, CollisionProp>,
    key_alloc: KeyAllocator,
    bounds: UniqueStore<CollisionObjectHandle>,
    types: UniqueStore<EntityType>,
    player: Key,
}

impl Store {
    fn new() -> Store {
        let mut state = Store {
            world: CollisionWorld::new(0.02),
            key_alloc: KeyAllocator::new(),
            bounds: UniqueStore::new(),
            types: UniqueStore::new(),
            player: Key::null(),
        };
        state.player = state.add((0, 0), 0, Cuboid::new((PLAYER_SIZE / 2).into_vector()), true, Player);
        state.player = state.add((0, 0), 0, Cuboid::new((PLAYER_SIZE / 2).into_vector()), true, Player);
        state
    }

    fn add(&mut self, pos: impl Into<Vector>, angle: impl Scalar, bounds: impl Shape<f32>, solid: bool, entity_type: EntityType) -> Key {
        let key = self.key_alloc.alloc();
        let pos: Vector = pos.into();
        let bounds = ShapeHandle::new(bounds);
        let query_type = match solid {
            true => GeometricQueryType::Contacts(0.02, 0.02),
            false => GeometricQueryType::Proximity(0.02),
        };
        let isometry = na::Isometry2::new(pos.into_vector(), angle.float());
        self.bounds.insert(key, self.world.add(isometry, bounds, CollisionGroups::new(), query_type, CollisionProp::Entity(key)));
        self.types.insert(key, entity_type);
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
        let world = &mut self.store.world;
        self.store.bounds.iter().for_each(|(_, bounds)| {
            let object = world.collision_object_mut(*bounds).unwrap();
            let position: Vector = object.position().translation.vector.into();
            let transform = na::Isometry2::new((position + Vector::new(0.01, 0.01)).into_vector(), 0.0);
            object.set_position(transform);
        });
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        use Background::*;
        window.clear(Color::BLACK)?;
        from(self.store.bounds.iter(), self.store.types.iter()).for_each(|(_, (bounds, ent_type))| {
            let color = match ent_type {
                Player => Color::BLUE,
            };
            let object = self.store.world.collision_object(*bounds).unwrap();
            let position: Vector = object.position().translation.vector.into();
            let area = Rectangle::new_sized((32, 32)).translate(position * PIXELS_PER_UNIT);
            window.draw(&area, Col(color));
        });
        Ok(())
    }
}

fn main() {
    run::<Game>("Rebound", Vector::new(960, 540), Settings::default());
}
