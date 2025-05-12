# Game Engine in Rust

This repository contains a simple 2D game engine written in Rust. The engine is designed to simulate basic physics, handle collisions, and render objects in a window using the `minifb` crate.

## Features

- **Physics Simulation**: 
  - Gravity
  - Air resistance
  - Ground drag
- **Collision Handling**:
  - Circle-based collision detection
  - Bounciness and damping factors
- **Rendering**:
  - Pixel-based rendering using a buffer
  - Adjustable window size
- **Input Handling**:
  - Keyboard input for object control

## Dependencies

The project uses the following Rust crates:
- [`anyhow`](https://crates.io/crates/anyhow): For error handling.
- [`minifb`](https://crates.io/crates/minifb): For window creation and rendering.

## How It Works

### Engine

The `Engine` struct manages the game loop, physics calculations, collision detection, and rendering. It maintains a list of game objects and updates them every frame.

### Game Objects

Game objects implement the `GameObject` trait, which defines methods for:
- Physics properties (e.g., weight, bounciness)
- Collision shape
- Rendering logic
- Input handling

### Example: Ball Object

The `Ball` struct is an example of a game object. It represents a circular object that can bounce around the window. The ball reacts to gravity, collisions, and keyboard input (e.g., moving left/right or jumping).

### Main Loop

The main loop:
1. Clears the display buffer.
2. Updates each game object's state (e.g., velocities, positions).
3. Handles collisions with the window boundaries.
4. Draws each object to the buffer.
5. Reflects the buffer changes in the window.

## Running the Project

1. Install Rust from [rust-lang.org](https://www.rust-lang.org/).
2. Clone this repository:
   ```sh
   git clone https://github.com/your-username/game-engine-rs.git
   cd game-engine-rs
   ```
3. Build and run the project:
   ```sh
   cargo run
   ```

## Controls

- **W**: Jump (if the ball is on the ground).
- **A**: Move left.
- **D**: Move right.
- **Escape**: Close the window.

## File Structure

```
.
├── Cargo.toml          # Project metadata and dependencies
├── src/
│   └── main.rs         # Main source code
└── target/             # Build artifacts (ignored by Git)
```

## Future Improvements

- Add support for more collision shapes (e.g., rectangles).
- Implement window resizing.
- Add more game objects and interactions.
- Improve rendering performance.

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.