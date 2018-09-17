extern crate nalgebra as na;
extern crate ncollide2d;
extern crate quicksilver;

use ncollide2d::{
    events::ContactEvent,
    shape::{Cuboid, Shape, ShapeHandle},
    world::{CollisionGroups, CollisionObject, CollisionObjectHandle, CollisionWorld, GeometricQueryType}
};
use quicksilver::{
    *,
    geom::{*, Shape as ShapeTrait},
    graphics::*,
    input::{*, Key as KeyboardKey},
    lifecycle::*,
    sound::*,
};

mod unique_store;
use unique_store::{Key, KeyAllocator, UniqueStore, join_key};

const PIXELS_PER_UNIT: f32 = 100.0;
const PLAYER_SIZE: Vector = Vector {
    x: 0.16,
    y: 0.16,
};

#[derive(Copy, Clone, Debug)]
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
    velocity: UniqueStore<Vector>,
    acceleration: UniqueStore<Vector>,
    friction: UniqueStore<f32>, // the fraction of the velocity to retain frame-over-frame
    embed: UniqueStore<Vector>, // the vector sum required to move the character out of all overlapping terrain
    velocity_cap: UniqueStore<Vector>,
}

impl Store {
    fn new() -> Store {
        Store {
            world: CollisionWorld::new(0.002),
            key_alloc: KeyAllocator::new(),
            bounds: UniqueStore::new(),
            types: UniqueStore::new(),
            velocity: UniqueStore::new(),
            acceleration: UniqueStore::new(),
            friction: UniqueStore::new(),
            embed: UniqueStore::new(),
            velocity_cap: UniqueStore::new(),
        }
    }

    fn create_physical_entity(&mut self, entity_type: EntityType) -> Key {
        let key = self.key_alloc.alloc();
        self.types.insert(key, entity_type);
        self.velocity.insert(key, Vector::ZERO);
        self.acceleration.insert(key, Vector::ZERO);
        self.friction.insert(key, 1.0);
        self.embed.insert(key, Vector::ZERO);
        key
    }

    fn create_collision_object(&mut self, pos: impl Into<Vector>, angle: impl Scalar, bounds: impl Shape<f32>, solid: bool, prop: CollisionProp) -> CollisionObjectHandle {
        let pos: Vector = pos.into();
        let bounds = ShapeHandle::new(bounds);
        let query_type = match solid {
            true => GeometricQueryType::Contacts(0.002, 0.002),
            false => GeometricQueryType::Proximity(0.002),
        };
        let isometry = na::Isometry2::new(pos.into_vector(), angle.float());
        self.world.add(isometry, bounds, CollisionGroups::new(), query_type, prop)
    }
}


struct Game {
    store: Store,
    player: Key,
    wall: Key,
}

impl State for Game {
    fn new() -> Result<Game> {
        let mut store = Store::new();
        let player = store.create_physical_entity(Player);
        let player_obj = store.create_collision_object((0, 0), 0, Cuboid::new((PLAYER_SIZE / 2).into_vector()), true, CollisionProp::Entity(player));
        store.bounds.insert(player, player_obj);
        store.friction[player] = 0.9;
        store.velocity_cap.insert(player, Vector::new(0.06, 0.15));
        let wall = store.create_physical_entity(Player);
        let wall_obj = store.create_collision_object((0.2, 0), 0, Cuboid::new((PLAYER_SIZE / 2).into_vector()), true, CollisionProp::Entity(wall));
        store.bounds.insert(wall, wall_obj);
        Ok(Game {
            store,
            player,
            wall
        })
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        let store = &mut self.store;
        // INPUT
        store.acceleration[self.player] = Vector::ZERO;
        if window.keyboard()[KeyboardKey::D].is_down() {
            store.acceleration[self.player].x += 0.003;
        }
        if window.keyboard()[KeyboardKey::A].is_down() {
            store.acceleration[self.player].x -= 0.003;
        }
        if window.keyboard()[KeyboardKey::Space] == ButtonState::Pressed {
            let a = store.bounds[self.player];
            let b = store.bounds[self.wall];
            let contact = store.world.contact_pair(a, b);
            println!("{:?}", contact.is_some());
        }
        // PHYSICS
        let world = &mut store.world;
        join_key(store.velocity.iter_mut(), store.acceleration.iter())
            .for_each(|(_, (velocity, acceleration))| *velocity += *acceleration );
        join_key(store.velocity.iter_mut(), store.friction.iter())
            .for_each(|(_, (velocity, friction))| *velocity *= *friction);
        join_key(store.velocity.iter_mut(), store.velocity_cap.iter())
            .for_each(|(_, (velocity, velocity_cap))| *velocity = velocity.clamp(-*velocity_cap, *velocity_cap));
        join_key(store.bounds.iter(), store.velocity.iter())
            .for_each(|(_, (bounds, velocity))| translate_obj(world.collision_object_mut(*bounds).unwrap(), *velocity));
        store.embed.iter_mut().for_each(|(_, embed)| *embed = Vector::ZERO);
        world.update();
        for event in world.contact_events() {
            match event {
                ContactEvent::Started(handle_a, handle_b) => {
                    let obj_a = world.collision_object(*handle_a).unwrap();
                    let obj_b = world.collision_object(*handle_b).unwrap();
                    match (obj_a.data(), obj_b.data()) {
                        (CollisionProp::Entity(key), CollisionProp::Terrain) => {
                            println!("{:?}", key);
                            // TODO: handle entity - terrain collisions
                        }
                        _ => ()
                    }
                }
                _ => ()
            }
        }
        join_key(store.bounds.iter_mut(), store.embed.iter_mut())
            .for_each(|(_, (bounds, embed))| translate_obj(world.collision_object_mut(*bounds).unwrap(), *embed));
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        use Background::*;
        window.clear(Color::BLACK)?;
        join_key(self.store.bounds.iter(), self.store.types.iter()).for_each(|(_, (bounds, ent_type))| {
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

fn translate_obj(object: &mut CollisionObject<f32, CollisionProp>, amount: Vector) {
    let position: Vector = object.position().translation.vector.into();
    let transform = na::Isometry2::new((position + amount).into_vector(), 0.0);
    object.set_position(transform);
}

fn main() {
    run::<Game>("Rebound", Vector::new(960, 540), Settings::default());
}
