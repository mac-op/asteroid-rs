mod game_object;

use std::f32::consts::{PI, TAU};
use rand::{Rng, RngCore, SeedableRng};
use rand::distributions::Standard;
use raylib::ffi::MouseButton::MOUSE_LEFT_BUTTON;
use raylib::prelude::*;
use crate::game_object::{Alien, AlienSize, Asteroid, AsteroidSize, ParticleType, Projectile, Ship, State, vector2_distance, vector2_rotate};

const SIZE: (i32, i32) = (1280, 960);
const SCALE: f32 = 38.0;
const SHIP_LINES: [Vector2; 5] = [
    Vector2::new(-0.4, -0.5),
    Vector2::new(0.0, 0.5),
    Vector2::new(0.4, -0.5),
    Vector2::new(0.3, -0.4),
    Vector2::new(-0.3, -0.4),
];

fn draw_lines(d: &mut RaylibDrawHandle, org: &Vector2, scale: f32, rot: f32, points: &Vec<Vector2>, connect: bool) {
    let transform = |p: &Vector2| -> Vector2 {
        let mut rotated = vector2_rotate(&p, rot);
        rotated.scale(scale);
        *org + rotated
    };

    let bound = if connect { points.len() } else { points.len() - 1 };
    for i in 0..bound {
        d.draw_line_ex(
            transform(&points[i]),
            transform(&points[(i+ 1) % points.len()]),
            2.5,
            Color::WHITE
        );
    }
}

fn reset_game(state: &mut State) {
    state.lives = 3;
    state.score = 0;

    reset_stage(state);
    reset_asteroids(state);
}

fn reset_stage(state: &mut State) {
    if state.ship.is_dead() {
        if state.lives == 0 {
            state.reset = true;
        } else { state.lives -= 1; }
    }
    state.ship = Ship {
        pos: Vector2{x: SIZE.0 as f32 * 0.5, y: SIZE.1 as f32 * 0.5},
        vel: Default::default(),
        rot: 0.0,
        death_time: 0.0,
    };
}

fn reset_asteroids(state:&mut State) {
    state.asteroids.clear();
    for _ in 0..(30 + state.score / 1500) {
        let angle = TAU * state.rand.sample::<f32, _>(Standard);
        let size = match state.rand.gen_range(0..3) {
            0 => AsteroidSize::SMALL,
            1 => AsteroidSize::MEDIUM,
            2 => AsteroidSize::BIG,
            _ => unreachable!()
        };
        state.asteroids_queue.push(
            Asteroid {
                pos: Vector2 {
                    x: state.rand.sample(Standard),
                    y: state.rand.sample(Standard)
                },
                vel: Vector2 {
                    x: f32::cos(angle),
                    y: f32::sin(angle)
                },
                size,
                seed: state.rand.next_u64(),
                remove: false,
            }
        );
    }
    state.stage_start = state.now;
}

fn hit_asteroid(state: &mut State, asteroid: &mut Asteroid, impact: Vector2) {
    state.score += asteroid.size.score();
    asteroid.remove = true;
    if asteroid.size == AsteroidSize::SMALL {
        return;
    }
    for _ in 0..2 {
        let dir = asteroid.vel.normalized();
        let size:AsteroidSize = match asteroid.size {
            AsteroidSize::MEDIUM => AsteroidSize::SMALL,
            AsteroidSize::BIG => AsteroidSize::MEDIUM,
            AsteroidSize::SMALL => unreachable!()
        };

        state.asteroids_queue.push(
            Asteroid{
                pos: asteroid.pos,
                vel: dir.scale_by(asteroid.size.vel_scale() * 2.2 * rand::random::<f32>())
                    + impact.scale_by(0.7),
                size,
                remove: false,
                seed: state.rand.next_u64()
            }
        );
    }
}

fn update(d: &RaylibDrawHandle, state: &mut State) {
    if state.asteroids.len() >100 || state.asteroids_queue.len() >100 {panic!()}

    if state.reset {
        state.reset = false;
        reset_game(state);
    }

    if !state.ship.is_dead() {
        const ROT_SPEED: f32 = 2f32;
        const SHIP_SPEED: f32 = 24f32;

        if d.is_key_down(KeyboardKey::KEY_A) {
            state.ship.rot -= state.delta * TAU * ROT_SPEED;
        }
        if d.is_key_down(KeyboardKey::KEY_D) {
            state.ship.rot += state.delta * TAU * ROT_SPEED;
        }
        let dir_angle: f32 = state.ship.rot + (PI * 0.5);
        let ship_dir: Vector2 = Vector2::new(f32::cos(dir_angle), f32::sin(dir_angle));

        if d.is_key_down(KeyboardKey::KEY_W) {
            state.ship.vel += ship_dir.scale_by(state.delta * SHIP_SPEED);
        }

        let drag: f32 = 0.015;
        state.ship.vel.scale(1.0 - drag);
        state.ship.pos += state.ship.vel;
        state.ship.pos.x = if state.ship.pos.x > SIZE.0 as f32 {state.ship.pos.x - SIZE.0 as f32} else {state.ship.pos.x};
        state.ship.pos.y = if state.ship.pos.y > SIZE.1 as f32 {state.ship.pos.y - SIZE.1 as f32} else {state.ship.pos.y};

        if d.is_key_pressed(KeyboardKey::KEY_SPACE) || d.is_mouse_button_pressed(MOUSE_LEFT_BUTTON) {
            state.projectiles.push(Projectile {
                pos: state.ship.pos + ship_dir.scale_by(38f32 * 0.5),
                vel: ship_dir.scale_by(10f32),
                ttl: 2.0,
                spawn: state.now,
                remove: false,
            });
            state.ship.vel += ship_dir.scale_by(-0.5);
        }

        for p in state.projectiles.iter_mut() {
            if !p.remove && (state.now - p.spawn) > 0.5 && vector2_distance(&state.ship.pos, &p.pos) < SCALE * 0.7 {
                p.remove = true;
                state.ship.death_time = state.now;
            }
        }
    }

    for item in state.asteroids_queue.iter() {
        state.asteroids.push(item.clone());
    }
    state.asteroids_queue.clear();

    let mut asteroids = std::mem::take(&mut state.asteroids);
    asteroids.retain_mut(|a |{
        a.pos += a.vel;
        a.pos.x = if a.pos.x > SIZE.0 as f32 {a.pos.x - SIZE.0 as f32} else {a.pos.x};
        a.pos.y = if a.pos.y > SIZE.1 as f32 {a.pos.y - SIZE.1 as f32} else {a.pos.y};
        let mut hit = false;
        let mut impact: Vector2 = Vector2{x: 0.0, y:0.0};

        if !state.ship.is_dead() && vector2_distance(&a.pos, &state.ship.pos) < a.size.size() * a.size.coll_scale() {
            state.ship.death_time = state.now;
            hit = true;
            impact = state.ship.vel.normalized();
        }

        for alien in state.aliens.iter_mut() {
            if !alien.remove && vector2_distance(&a.pos, &alien.pos) < a.size.coll_scale() * a.size.size() {
                alien.remove = true;
                hit = true;
                impact = state.ship.vel.normalized();
            }
        }

        for p in state.projectiles.iter_mut() {
            if !p.remove && vector2_distance(&a.pos, &p.pos) < a.size.coll_scale() * a.size.size() {
                p.remove = true;
                hit = true;
                impact = p.vel.normalized();
            }
        }
        hit_asteroid(state, a, impact);
        !a.remove
    });
    state.asteroids = asteroids;

    state.particles.retain_mut(|p| {
        p.pos += p.vel;
        p.pos.x = if p.pos.x > SIZE.0 as f32 {p.pos.x - SIZE.0 as f32} else {p.pos.x};
        p.pos.y = if p.pos.y > SIZE.1 as f32 {p.pos.y - SIZE.1 as f32} else {p.pos.y};

        if p.ttl > state.delta {
            p.ttl -= state.delta;
            false
        } else { true }
    });

    let mut aliens = std::mem::take(& mut state.aliens);
    aliens.retain_mut(|a| {
        for p in state.projectiles.iter_mut() {
            if !p.remove && (state.now - p.spawn) > 0.15 && vector2_distance(&a.pos, &p.pos) < a.size.coll_size() {
                p.remove = true;
                a.remove = true;
            }
        };

        if !a.remove && vector2_distance(&a.pos, &state.ship.pos) < a.size.coll_size() {
            a.remove = true;
            state.ship.death_time = state.now;
        }

        if !a.remove {
            if state.now - a.last_dir > a.size.dir_change_time() {
                a.last_dir = state.now;
                let angle = TAU * rand::random::<f32>();
                a.dir = Vector2{ x: f32::cos(angle), y: f32::sin(angle)};
            }
            a.pos += a.dir.scale_by(a.size.speed());
            a.pos.x = if a.pos.x > SIZE.0 as f32 {a.pos.x - SIZE.0 as f32} else {a.pos.x};
            a.pos.y = if a.pos.y > SIZE.1 as f32 {a.pos.y - SIZE.1 as f32} else {a.pos.y};

            if state.now - a.last_shot > a.size.shot_time() {
                a.last_shot = state.now;
                let dir = (state.ship.pos - a.pos).normalized();
                state.projectiles.push(
                  Projectile {
                      pos: a.pos + dir.scale_by(SCALE * 0.55),
                      vel: dir.scale_by(6.0),
                      ttl: 2.0,
                      spawn: state.now,
                      remove: false,
                  }
                );
            }
        }
        !a.remove
    });
    state.aliens = aliens;

    if state.ship.is_dead() && state.now - state.ship.death_time > 3.0 {
        reset_stage(state);
    }

    if state.last_score / 5000 != state.score / 5000 {
        state.aliens.push( Alien{
            pos: Vector2::new(
                if rand::random() {0.0} else {SIZE.0 as f32 - SCALE},
                rand::random::<f32>() * SIZE.1 as f32,
            ),
            dir: Vector2::new(0.0, 0.0),
            size: AlienSize::BIG,
            remove: false,
            last_shot: 0.0,
            last_dir: 0.0,
        });
    }
    if state.last_score / 8000 != state.score / 8000 {
        state.aliens.push( Alien{
            pos: Vector2::new(
                if rand::random() {0.0} else {SIZE.0 as f32 - SCALE},
                rand::random::<f32>() * SIZE.1 as f32,
            ),
            dir: Vector2::new(0.0, 0.0),
            size: AlienSize::SMALL,
            remove: false,
            last_shot: 0.0,
            last_dir: 0.0,
        });
    }

    state.last_score = state.score;
}

fn main() {
    let (mut rl, thread) = init()
        .size(SIZE.0, SIZE.1)
        .title("Hello World")
        .build();

    let mut state = State::init();
    reset_game(& mut state);
    rl.set_target_fps(60);
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        state.delta = d.get_frame_time();
        state.now += state.delta;

        update(&d, & mut state);
        render(& mut d, & state);
        state.frame += 1;
    }
}

fn render(d: & mut RaylibDrawHandle, state: &State) {
    // if state.asteroids.len() >100 || state.asteroids_queue.len() >100 {panic!()}
    for i in 0..state.lives {
        draw_lines(d, &Vector2::new(SCALE + i as f32, SCALE), SCALE, -PI,
        &SHIP_LINES.to_vec(), true);
    }
    if !state.ship.is_dead() {
        draw_lines(d, &state.ship.pos, SCALE, state.ship.rot, &SHIP_LINES.to_vec(), true);
    }

    if d.is_key_down(KeyboardKey::KEY_W) && ((state.now * 20.0) as i32 % 2 == 0) {
        draw_lines(d, &state.ship.pos, SCALE, state.ship.rot,
                   &vec![
                       Vector2::new(-0.3, -0.4),
                       Vector2::new(0.0, -1.0),
                       Vector2::new(0.3, -0.4)
                   ], true);
    }

    for asteroid in & state.asteroids {
        draw_asteroid(d, & asteroid.pos, & asteroid.size, &asteroid.seed);
    }
    for alien in & state.aliens {
        draw_alien(d, & alien.pos, & alien.size);
    }
    for particle in & state.particles {
        match particle.values {
            ParticleType::LINE {rot, length} =>  {
                draw_lines(d, &particle.pos, length, rot,
                & vec![Vector2::new(-0.5, 0.0), Vector2::new(0.5, 0.0)], true);
            },
            ParticleType::DOT { radius } => {
                d.draw_circle_v(particle.pos.clone(), radius, Color::WHITE);
            }
        }
    }
    for projectile in & state.projectiles {
        d.draw_circle_v(projectile.pos, f32::max(SCALE * 0.05, 1.0), Color::WHITE);
    }
}

fn draw_alien(d: &mut RaylibDrawHandle, p: &Vector2, s: &AlienSize) {
    let scale: f32 = match s {
        AlienSize::SMALL => 0.5,
        AlienSize::BIG => 1.0
    };
    draw_lines(d, p, SCALE * scale, 0.0, &vec![
        Vector2::new(-0.5, 0.0),
        Vector2::new(-0.3, 0.3),
        Vector2::new(0.3, 0.3),
        Vector2::new(0.5, 0.0),
        Vector2::new(0.3, -0.3),
        Vector2::new(-0.3, -0.3),
        Vector2::new(-0.5, 0.0),
        Vector2::new(0.5, 0.0),
    ], false);

    draw_lines(d, p, SCALE * scale, 0.0, &vec![
        Vector2::new(-0.2, -0.3),
        Vector2::new(-0.1, -0.5),
        Vector2::new(0.1, -0.5),
        Vector2::new(0.2, -0.3),
    ], false);
}

fn draw_asteroid(d: & mut RaylibDrawHandle, pos: &Vector2, size:&AsteroidSize, seed: &u64) {
    let mut r = rand::rngs::StdRng::seed_from_u64(*seed);
    let mut points: Vec<Vector2> = Vec::with_capacity(16);
    let n = r.gen_range(8..15);

    for i in 0..n {
        let mut radius = 0.3 + (0.2 * r.sample::<f32, _>(Standard));
        if r.sample::<f32, _>(Standard) < 0.2 { radius -= 0.2; }

        let angle = (i as f32) * (TAU / n as f32) + (PI * 0.125 * r.sample::<f32, _>(Standard));
        points.push(Vector2::new(f32::cos(angle), f32::sin(angle)).scale_by(radius));
    }
    draw_lines(d, &pos, size.size(), 0.0, & points, true);
}
