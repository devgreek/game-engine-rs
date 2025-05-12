use minifb::{ Key, Window, WindowOptions, ScaleMode };
use anyhow;

const WIDTH: usize = 640;
const HEIGHT: usize = 360;
const GROUND_DRAG_FACTOR: f64 = 0.1;
const GRAVITY: f64 = 0.5;
const AIR_RESISTANCE_FACTOR: f64 = 0.01;
const DT: f64 = 1.0;
const FPS: u64 = 60;

#[derive(Clone, Default)]
pub struct XYPair {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone)]
pub struct ObjectInfo {
    // in case we want to support window resizing in the
    // future, the objects might want to know the size.
    // this is however a bad example, because it doesn't
    // need to be set per game object, so let's fix that in part 2
    pub window_size: WindowSize,
}

#[derive(Default)]
pub struct GameObjectCommon {
    pub coords: XYPair,
    pub velocities: XYPair,
    pub object_info: Option<ObjectInfo>,
}

pub enum CollisionShape {
    Circle(f64),
}
pub enum CollisionType {
    Circle,
    Rectangle,
}

pub struct Engine {
    window: Option<Window>,
    buffer: Vec<u32>,
    window_size: WindowSize,
    objects: Vec<Box<dyn GameObject>>,
}

impl Engine {
    pub fn new(window_size: &WindowSize) -> Result<Self, anyhow::Error> {
        Ok(Self {
            buffer: vec![0; window_size.width * window_size.height],
            window: None,
            window_size: window_size.clone(),
            objects: Vec::new(),
        })
    }

    pub fn add_game_object(&mut self, game_object: impl GameObject + 'static) {
        self.objects.push(Box::new(game_object))
    }

    fn calc_velocities(object: &mut Box<dyn GameObject>) {
        let mut velocities = object.common().velocities.clone();

        // apply gravity
        let gravity = GRAVITY * object.weight_factor() * DT;
        velocities.y += gravity;

        // apply air drag
        velocities.x *= 1.0 - AIR_RESISTANCE_FACTOR * DT;
        velocities.y *= 1.0 - AIR_RESISTANCE_FACTOR * DT;

        object.common().velocities = velocities;
    }

    fn apply_velocities(object: &mut Box<dyn GameObject>) {
        let common = object.common();
        let coords = common.coords.clone();
        let velocities = common.velocities.clone();

        object.common().coords = XYPair {
            x: coords.x + velocities.x,
            y: coords.y + velocities.y,
        };
    }

    fn update_object_info(window_size: &WindowSize, object: &mut Box<dyn GameObject>) {
        object.common().object_info = Some(ObjectInfo {
            window_size: window_size.clone(),
        });
    }

    fn draw(buffer: &mut Vec<u32>, window_size: &WindowSize, object: &mut Box<dyn GameObject>) {
        let raster_vecs = object.draw();

        let common = object.common();
        let coords = &common.coords;

        Engine::draw_at(buffer, window_size.width, window_size.height, raster_vecs, coords);
    }

    fn collision_checks(window_size: &WindowSize, object: &mut Box<dyn GameObject>) {
        match object.collision_shape() {
            CollisionShape::Circle(radius) => {
                let mut coords = object.common().coords.clone();
                let mut velocities = object.common().velocities.clone();
                let diameter = 2.0 * radius;

                let on_ground = if coords.y + diameter >= (window_size.height as f64) {
                    true
                } else {
                    false
                };

                let on_x_collision = |velocities: &mut XYPair| {
                    velocities.x = -velocities.x * object.bounciness();
                };

                let on_y_collision = |velocities: &mut XYPair| {
                    velocities.y = -velocities.y * object.bounciness();

                    // if we're just rolling on the ground, apply ground drag
                    if on_ground && velocities.y.abs() <= 1.0 {
                        velocities.x -= velocities.x * GROUND_DRAG_FACTOR;
                    }
                };

                // x axis window collision
                if coords.x <= 0.0 {
                    coords.x = 0.0;
                    on_x_collision(&mut velocities);
                }
                if coords.x + diameter > (window_size.width as f64) {
                    coords.x = (window_size.width as f64) - diameter;
                    on_x_collision(&mut velocities);
                }

                // y axis window collision
                if coords.y - diameter < 0.0 {
                    coords.y = diameter;
                    on_y_collision(&mut velocities);
                }
                if coords.y + diameter > (window_size.height as f64) {
                    coords.y = (window_size.height as f64) - diameter;
                    on_y_collision(&mut velocities);
                }

                object.common().coords = coords;
                object.common().velocities = velocities;
            }
        }
    }

    fn draw_at(
        buffer: &mut Vec<u32>,
        buffer_width: usize,
        buffer_height: usize,
        raster_vecs: Vec<Vec<u32>>,
        coords: &XYPair
    ) {
        let object_width = raster_vecs
            .iter()
            .map(|row| row.len())
            .max()
            .unwrap_or(0);

        for (dy, row) in raster_vecs.iter().enumerate() {
            for dx in 0..object_width {
                let x = (coords.x + (dx as f64)) as usize;
                let y = (coords.y + (dy as f64)) as usize;

                // make sure this is not out of the buffer
                if x < buffer_width && y < buffer_height {
                    let index = y * buffer_width + x;

                    let maybe_pixel = row.get(dx).cloned();
                    if let Some(pixel) = maybe_pixel {
                        buffer[index] = pixel;
                    }
                }
            }
        }
    }

    pub fn run(&mut self, window_title: &str) -> Result<(), anyhow::Error> {
        self.window = Some(
            Window::new(
                window_title,
                self.window_size.width,
                self.window_size.height,
                WindowOptions {
                    scale_mode: ScaleMode::AspectRatioStretch,
                    ..WindowOptions::default()
                }
            )?
        );

        let duration_per_frame = std::time::Duration::from_secs(1) / FPS.try_into()?;
        self.window.as_mut().unwrap().limit_update_rate(Some(duration_per_frame));

        while
            self.window.as_ref().unwrap().is_open() &&
            !self.window.as_ref().unwrap().is_key_down(Key::Escape)
        {
            let keys = self.window.as_ref().unwrap().get_keys();

            // clear the display buffer
            self.buffer.iter_mut().for_each(|p| {
                *p = 0;
            });

            for object in self.objects.iter_mut() {
                // re-calculate the velocities of the object
                Engine::calc_velocities(object);

                // apply the velocities to the coordinates
                Engine::apply_velocities(object);

                // perform collision checks with the window
                Engine::collision_checks(&self.window_size, object);

                // update the game object's info
                Engine::update_object_info(&self.window_size, object);

                // allow the object to react to pressed keys
                object.handle_input(&keys);

                // draw the object on the buffer at it's coords
                Engine::draw(&mut self.buffer, &self.window_size, object);
            }

            // reflect the display buffer changes
            self.window
                .as_mut()
                .unwrap()
                .update_with_buffer(&self.buffer, self.window_size.width, self.window_size.height)?;
        }

        Ok(())
    }
}

pub const DEFAULT_COLLISION_DAMPING_FACTOR: f64 = 0.8;
pub const DEFAULT_COLLISION_DAMPING_FACTOR_RECTANGLE: f64 = 0.5;

pub trait GameObject {
    fn common(&mut self) -> &mut GameObjectCommon;

    fn weight_factor(&self) -> f64;

    fn bounciness(&self) -> f64 {
        DEFAULT_COLLISION_DAMPING_FACTOR
    }

    fn collision_shape(&self) -> CollisionShape;

    fn draw(&self) -> Vec<Vec<u32>>;

    fn handle_input(&mut self, _keys: &[Key]) {}
}

#[derive(Clone)]
pub struct WindowSize {
    pub height: usize,
    pub width: usize,
}

pub struct Ball {
    radius: f64,
    diameter: f64,
    color: u32,

    common: GameObjectCommon,
}

impl GameObject for Ball {
    fn common(&mut self) -> &mut GameObjectCommon {
        &mut self.common
    }

    fn weight_factor(&self) -> f64 {
        0.8
    }

    fn bounciness(&self) -> f64 {
        0.6
    }

    fn collision_shape(&self) -> CollisionShape {
        CollisionShape::Circle(self.radius)
    }

    fn draw(&self) -> Vec<Vec<u32>> {
        self.draw()
    }

    fn handle_input(&mut self, keys: &[Key]) {
        self.handle_input(keys)
    }
}

impl Ball {
    pub fn new(coords: XYPair, radius: f64, color_hex: &str) -> Self {
        let diameter = radius * 2.0;

        // convert hex color to u32, or default to white
        let color = u32::from_str_radix(&color_hex[1..], 16).unwrap_or(0xffffff);

        let common = GameObjectCommon {
            coords,
            ..GameObjectCommon::default()
        };

        Self {
            color,
            radius,
            diameter,

            common,
        }
    }

    pub const KB_X_BOOST: f64 = 0.2;
    pub const KB_Y_BOOST: f64 = 16.0;

    fn handle_input(&mut self, keys: &[Key]) {
        if keys.contains(&Key::A) {
            self.common.velocities.x -= Self::KB_X_BOOST;
        }

        if keys.contains(&Key::D) {
            self.common.velocities.x += Self::KB_X_BOOST;
        }

        // jump if we are on the ground AND have 0 or lesser y velocity
        if keys.contains(&Key::W) {
            if let Some(info) = &self.common.object_info {
                let window_height = info.window_size.height as f64;
                if
                    self.common.velocities.y < 0.0 &&
                    self.common.coords.y + self.diameter == window_height
                {
                    self.common.velocities.y -= Self::KB_Y_BOOST;
                }
            }
        }
    }

    fn draw(&self) -> Vec<Vec<u32>> {
        let mut raster =
            vec![
         vec![
             0; self.diameter as usize
            ]; self.diameter as usize
        ];

        let h = self.radius;
        let k = self.radius;

        for y in 0..self.diameter as usize {
            for x in 0..self.diameter as usize {
                let dx = ((x as f64) - h).abs();
                let dy = ((y as f64) - k).abs();
                if (dx * dx + dy * dy).sqrt() <= self.radius {
                    raster[y][x] = self.color;
                }
            }
        }

        raster
    }
}

fn main() -> Result<(), anyhow::Error> {
  let window_size = WindowSize {width: 800, height: 600};
  let mut engine = Engine::new(&window_size)?;
  
  let radius = 24.0;
  let ball_coords = XYPair {
    x: (&window_size.width / 2) as f64 - radius,
    y: (&window_size.height / 2) as f64 - radius,
  };
  let ball = Ball::new(ball_coords, radius, "#cf5353");

  engine.add_game_object(ball);
  engine.run("Bouncy Ball")
}