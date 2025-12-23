use nalgebra::*;

// --- 1. Define the Rocket State ---
// This struct holds the "snapshot" of the rocket at any given moment.
#[derive(Debug, Clone)]
struct RocketState {
    time: f64,
    position: Vector3<f64>,
    velocity: Vector3<f64>,
    acceleration: Vector3<f64>,
    orientation: UnitQuaternion<f64>,
    angular_velocity: Vector3<f64>,
}

impl RocketState {
    fn new() -> Self {
        Self {
            time: 0.0,
            position: Vector3::new(0.0, 0.0, 0.0), // Start at origin
            velocity: Vector3::new(0.0, 0.0, 0.0),
            acceleration: Vector3::new(0.0, 0.0, 0.0),
            orientation: UnitQuaternion::new_normalize(Quaternion::new(1.0, 0.0, 0.0, 0.0)), // Pointing "up" (local Z aligned with world Z)
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
        }
    }
}

// --- 2. The Simulation Engine ---
struct RocketSimulation {
    history: Vec<RocketState>, // Stores the entire flight path
    mass: f64,
    dt: f64,
}

impl RocketSimulation {
    fn new(mass: f64, dt: f64) -> Self {
        let initial_state = RocketState::new();
        Self {
            history: vec![initial_state],
            mass,
            dt,
        }
    }

    // Run the simulation for a specific duration
    fn run(&mut self, duration: f64) {
        let steps = (duration / self.dt) as usize;

        for _ in 0..steps {
            let prev = self.history.last().unwrap();
            let mut next = prev.clone();
            
            // --- A. Calculate Forces & Torques ---
            // 1. Gravity (World Space) pointing down -Z
            let gravity = Vector3::new(0.0, 0.0, -9.81) * self.mass;

            // 2. Thrust (Local Space) pointing up +Z relative to the rocket
            // simple logic: Thrust for first 10 seconds, then coast
            let thrust_force = if prev.time < 10.0 {
                25.0 * self.mass // Enough to overcome gravity
            } else {
                0.0
            };
            
            // Convert local thrust to world space using current orientation
            let thrust_world = prev.orientation * Vector3::new(0.0, 0.0, thrust_force);

            // 3. Torque (to tilt the rocket slightly)
            // Apply a small torque around X axis between t=1s and t=2s to start a turn
            let torque = if prev.time > 1.0 && prev.time < 2.0 {
                Vector3::new(0.1, 0.0, 0.0)
            } else {
                Vector3::new(0.0, 0.0, 0.0)
            };

            // --- B. Physics Integration (Semi-Implicit Euler) ---
            
            // Linear Motion
            let total_force = gravity + thrust_world;
            next.acceleration = total_force / self.mass;
            next.velocity += next.acceleration * self.dt;
            next.position += next.velocity * self.dt;

            // Rotational Motion
            // Angular Accel = Torque / Inertia (Simplified as Identity matrix for this example)
            let angular_accel = torque; 
            next.angular_velocity += angular_accel * self.dt;

            // Update Quaternion Orientation
            // q_new = q_old + 0.5 * (ang_vel * q_old) * dt
            let rotation_delta = UnitQuaternion::from_euler_angles(
                next.angular_velocity.x * self.dt,
                next.angular_velocity.y * self.dt,
                next.angular_velocity.z * self.dt,
            );
            next.orientation = rotation_delta * prev.orientation;

            // Advance time
            next.time += self.dt;

            // Store state
            self.history.push(next.clone());
            
            // Stop if we hit the ground
            if next.position.z < 0.0 {
                println!("Rocket crashed/landed at t={:.2}", next.time);
                break;
            }
        }
    }

    // --- C. Accessor Method ---
    // Get the state at any specific time t
    fn get_state_at(&self, time: f64) -> Option<&RocketState> {
        // Find the index that corresponds to the time
        let index = (time / self.dt) as usize;
        if index < self.history.len() {
            Some(&self.history[index])
        } else {
            None
        }
    }
}

fn main() {
    // 1. Setup Simulation (1000kg rocket, 0.01s time step)
    let mut sim = RocketSimulation::new(1000.0, 0.01);

    // 2. Run for 20 seconds
    println!("Simulating launch...");
    sim.run(20.0);

    // 3. Query specific data points
    // Example: What was happening at 5.0 seconds?
    if let Some(state) = sim.get_state_at(5.0) {
        println!("--- Status at t=5.0s ---");
        println!("Position (XYZ): {:.2}, {:.2}, {:.2}", state.position.x, state.position.y, state.position.z);
        println!("Vertical Velocity: {:.2} m/s", state.velocity.z);
        println!("Orientation (Quaternion): {:?}", state.orientation.coords);
    }

    // Example: What was happening at 12.0 seconds (Engine cutoff)?
    if let Some(state) = sim.get_state_at(12.0) {
        println!("\n--- Status at t=12.0s ---");
        println!("Vertical Accel: {:.2} m/s^2 (Gravity only)", state.acceleration.z);
        println!("Angular Vel: {:.4} rad/s", state.angular_velocity.norm());
    }
}