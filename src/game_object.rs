use rand::rngs::StdRng;
use rand::SeedableRng;
use raylib::math::Vector2;

const SCALE: f32 = 38.0;

#[derive(Debug)]
pub struct Ship {
    pub pos: Vector2,
    pub vel: Vector2,
    pub rot: f32,
    pub death_time: f32,
}

#[derive(Clone)]
pub struct Asteroid {
    pub pos: Vector2,
    pub vel: Vector2,
    pub size: AsteroidSize,
    pub seed: u64,
    pub remove: bool,
}

pub struct Alien {
    pub pos: Vector2,
    pub dir: Vector2,
    pub size: AlienSize,
    pub remove: bool,
    pub last_shot: f32,
    pub last_dir: f32
}

pub struct Particle {
    pub pos: Vector2,
    pub vel: Vector2,
    pub ttl: f32,
    pub values: ParticleType
}

#[derive(Default)]
pub struct Projectile {
    pub pos: Vector2,
    pub vel: Vector2,
    pub ttl: f32,
    pub spawn: f32,
    pub remove: bool,
}

pub struct State {
    pub now: f32,
    pub delta: f32,
    pub stage_start: f32,
    pub ship: Ship,
    pub asteroids: Vec<Asteroid>,
    pub asteroids_queue: Vec<Asteroid>,
    pub particles: Vec<Particle>,
    pub projectiles: Vec<Projectile>,
    pub aliens: Vec<Alien>,
    pub rand: StdRng,
    pub lives: usize,
    pub last_score: usize,
    pub(crate) score: usize,
    pub reset: bool,
    last_bloop: usize,
    bloop: usize,
    pub frame: usize,
}

#[derive(Clone, PartialEq)]
pub enum AsteroidSize { SMALL, MEDIUM, BIG }
pub enum AlienSize {SMALL, BIG}
pub enum ParticleType {
    LINE { rot: f32, length: f32 },
    DOT { radius: f32 },
}

impl AsteroidSize {
    pub(crate) fn score(&self) -> usize {
        match self {
            AsteroidSize::SMALL => 20,
            AsteroidSize::MEDIUM => 50,
            AsteroidSize::BIG => 100
        }
    }

    pub fn size(&self) -> f32 {
        match self {
            AsteroidSize::SMALL => SCALE * 0.8,
            AsteroidSize::MEDIUM => SCALE * 1.4,
            AsteroidSize::BIG => SCALE * 3.0
        }
    }

    pub fn coll_scale(&self) -> f32 {
        match self {
            AsteroidSize::SMALL => 1.0,
            AsteroidSize::MEDIUM => 0.6,
            AsteroidSize::BIG => 0.4
        }
    }

    pub fn vel_scale(&self) -> f32 {
        match self {
            AsteroidSize::SMALL => 3.0,
            AsteroidSize::MEDIUM => 1.8,
            AsteroidSize::BIG => 0.75
        }
    }
}

impl AlienSize {
    pub fn coll_size(&self) -> f32 {
        match self {
            AlienSize::SMALL => SCALE * 0.5,
            AlienSize::BIG => SCALE * 0.8
        }
    }

    pub fn dir_change_time(&self) ->f32 {
        match self {
            AlienSize::BIG => 0.85,
            AlienSize::SMALL => 0.35,
        }
    }

    pub fn shot_time(&self) -> f32 {
        match self {
            AlienSize::SMALL => 0.5,
            AlienSize::BIG => 1.25
        }
    }

    pub fn speed(&self) -> f32 {
        match self {
            AlienSize::SMALL => 3.0,
            AlienSize::BIG => 6.0
        }
    }
}

impl Ship {
    fn init() -> Ship {
        Ship {
            pos: Vector2::new((640 * 2) as f32 * 0.5, (480 * 2) as f32 * 0.5 ),
            vel: Default::default(),
            rot: 0.0,
            death_time: 0.0,
        }
    }

    pub fn is_dead(&self) -> bool {
        self.death_time != 0f32
    }
}

impl State {
    pub fn init() -> State {
        State {
            now: 0.0,
            delta: 0.0,
            stage_start: 0.0,
            ship: Ship::init(),
            asteroids: vec![],
            asteroids_queue: vec![],
            particles: vec![],
            projectiles: vec![],
            aliens: vec![],
            rand: StdRng::from_entropy(),
            lives: 0,
            last_score: 0,
            score: 0,
            reset: false,
            last_bloop: 0,
            bloop: 0,
            frame: 0,
        }
    }
}

pub fn vector2_rotate(v: &Vector2, angle: f32) -> Vector2 {
    let cos_res: f32 = f32::cos(angle);
    let sin_res: f32 = f32::sin(angle);
    Vector2::new(
        v.x * cos_res - v.y * sin_res,
        v.x * sin_res + v.y * cos_res
    )
}

pub fn vector2_distance(v: &Vector2, w: &Vector2) -> f32 {
    ((v.x - w.x) * (v.x - w.x) + (v.y - w.y) * (v.y - w.y)).sqrt()
}