use std::fmt::format;

use filter::kalman;
use kiss3d::egui::{self, Align2};
use kiss3d::window::Window;
use kiss3d::{camera::ArcBall, light::Light};
use nalgebra::*;
use rand_distr::{Distribution, Normal};

#[derive(Debug, Clone, Copy, Default)]
struct RocketState {
    time: f32,
    position: Point3<f32>,
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
    orientation: UnitQuaternion<f32>,
    angular_velocity: Vector3<f32>,
}

struct RocketSimulation {
    history: Vec<RocketState>,
    mass: f32,
    dt: f32,
    launch_angle: f32,
    sensor_noise: f32,
}

impl RocketSimulation {
    fn new(mass: f32, dt: f32) -> Self {
        let initial_state = RocketState::default();
        Self {
            history: vec![initial_state],
            mass,
            dt,
            launch_angle: 10.0f32,
            sensor_noise: 0.0f32,
        }
    }

    fn run(&mut self) {
        self.history.clear();
        let mut initial_state = RocketState {
            time: 0.0,
            position: Point3::origin(),
            velocity: Vector3::zeros(),     // Body Frame (0,0,0)
            acceleration: Vector3::zeros(), // Body Frame (0,0,0)
            orientation: UnitQuaternion::from_euler_angles(
                -self.launch_angle.to_radians(),
                0.0,
                0.0,
            ),
            angular_velocity: Vector3::zeros(),
        };
        self.history.push(initial_state);

        loop {
            let prev = self.history.last().cloned().unwrap_or_else(|| RocketState {
                time: 0.0,
                position: Point3::origin(),
                velocity: Vector3::zeros(),
                acceleration: Vector3::zeros(),
                orientation: UnitQuaternion::identity(),
                angular_velocity: Vector3::zeros(),
            });

            let mut next = prev.clone();

            // 1. Recover World Frame Velocity
            // We must rotate the stored Body Frame velocity into World Frame
            // to correctly update the World Frame position.
            let velocity_world_prev = prev.orientation * prev.velocity;

            // --- Forces ---

            // Gravity is constant in World Frame (Z-down)
            let gravity_world = Vector3::new(0.0, 0.0, -9.81) * self.mass;

            // Thrust is generated in Body Frame (Local Z-up), then rotated to World
            let thrust_mag = if prev.time < 1.0 {
                25.0 * self.mass
            } else {
                0.0
            };
            // Rotate local thrust vector (0, 0, magnitude) by orientation to get world vector
            let thrust_world = prev.orientation * Vector3::new(0.0, 0.0, thrust_mag);

            // Torque is in Body Frame (Standard for control inputs)
            let torque_local = if prev.time > 1.0 && prev.time < 2.0 {
                Vector3::new(0.1, 0.0, 0.0)
            } else {
                Vector3::new(0.0, 0.0, 0.0)
            };

            // --- Integration (World Frame) ---

            let total_force_world = gravity_world + thrust_world;
            let accel_world = total_force_world / self.mass;

            // Update Linear State (in World Frame)
            let velocity_world_next = velocity_world_prev + accel_world * self.dt;
            let position_next = prev.position + velocity_world_next * self.dt;

            // Update Angular State
            let angular_accel = torque_local; // Assuming Inertia is identity for simplicity
            next.angular_velocity += angular_accel * self.dt;

            let rotation_delta = UnitQuaternion::from_euler_angles(
                next.angular_velocity.x * self.dt,
                next.angular_velocity.y * self.dt,
                next.angular_velocity.z * self.dt,
            );
            next.orientation = rotation_delta * prev.orientation;

            // --- Storage (Convert back to Body Frame) ---

            next.time += self.dt;
            next.position = position_next;

            // Convert World Acceleration -> Body Acceleration
            // Rotated by the inverse of the orientation
            next.acceleration = next.orientation.inverse() * accel_world;

            // Convert World Velocity -> Body Velocity
            // This is crucial: if the rocket turns 90 degrees, the body-frame velocity
            // changes completely relative to the nose cone, even if world velocity is constant.
            next.velocity = next.orientation.inverse() * velocity_world_next;

            self.history.push(next.clone());

            if next.position.z < 0.0 {
                break;
            }
        }
    }
}

fn simulate_filter(sim: &mut RocketSimulation, filter: &mut kalman::LKF<10>) -> (Vec<RocketState>, f32) {
    sim.run();

    let mut rng = rand::rng();
    let normal = Normal::new(0.0f32, sim.sensor_noise).expect("Invalid params");
    let mut estimates: Vec<RocketState> = vec![];

    for ground_truth in &sim.history {
        // 1. Initialize or Clone the previous estimate
        let is_initial = estimates.is_empty();
        let mut new_state = estimates.last().cloned().unwrap_or(*ground_truth);

        // 2. Generate Sensor Noise
        let noise_accel = Vector3::new(
            normal.sample(&mut rng),
            normal.sample(&mut rng),
            normal.sample(&mut rng),
        );
        let noise_gyro = Vector3::new(
            normal.sample(&mut rng),
            normal.sample(&mut rng),
            normal.sample(&mut rng),
        );

        // 3. Create "Measured" Forces (Sensor Readings)
        let measured_accel_body = ground_truth.acceleration + noise_accel;
        let measured_omega_body = ground_truth.angular_velocity + noise_gyro;

        if is_initial {
            // Initialize first state with noisy measurements but no integration
            new_state.acceleration = measured_accel_body;
            new_state.angular_velocity = measured_omega_body;
            estimates.push(new_state);
            continue;
        }

        // 4. Dead Reckoning Integration

        let vel_world_prev = new_state.orientation * new_state.velocity;
        let accel_world = new_state.orientation * measured_accel_body;

        // C. Integrate Linear State
        let vel_world_next = vel_world_prev + accel_world * sim.dt;
        new_state.position += vel_world_next * sim.dt;
        
        // D. Integrate Angular State
        let rotation_delta = UnitQuaternion::from_euler_angles(
            measured_omega_body.x * sim.dt,
            measured_omega_body.y * sim.dt,
            measured_omega_body.z * sim.dt,
        );
        new_state.orientation = rotation_delta * new_state.orientation;

        // E. Store Body Frame Velocity
        new_state.velocity = new_state.orientation.inverse() * vel_world_next;

        // F. Update State Metadata
        new_state.time += sim.dt;
        new_state.acceleration = measured_accel_body;
        new_state.angular_velocity = measured_omega_body;

        estimates.push(new_state);
    }

    // RMSE
    let mse: f32 = estimates
        .iter()
        .zip(sim.history.iter())
        .map(|(est, gt)| (est.position - gt.position).norm_squared())
        .sum::<f32>()
        / estimates.len() as f32;

    let rmse = mse.sqrt();

    (estimates, rmse)
}

#[kiss3d::main]
async fn main() {
    let mut window = Window::new("Interactive Filter Simulator");
    let mut sim = RocketSimulation::new(1000.0, 0.01);
    let mut filter = kalman::LKF::<10>::new();
    let (mut estimates, mut rmse) = simulate_filter(&mut sim, &mut filter);

    window.set_light(Light::StickToCamera);

    // --- SETUP CAMERA ---
    let eye = Point3::new(-40.0, -40.0, 20.0);
    let at = Point3::origin();
    let mut camera = ArcBall::new(eye, at);
    camera.set_up_axis(*Vector3::z_axis());

    let mut sample_frequency = 100;
    let mut rmse = 0.0f32;

    while window.render_with_camera(&mut camera).await {
        window.draw_ui(|ctx| {
            egui::Window::new("Stats")
                .default_width(300.0)
                .anchor(Align2::LEFT_TOP, [-10.0, 10.0])
                .show(ctx, |ui| {
                    ui.label(format!("Error (RMS): {}", rmse));
                });

            egui::Window::new("Controls")
                .default_width(300.0)
                .anchor(Align2::RIGHT_TOP, [-10.0, 10.0])
                .show(ctx, |ui| {
                    ui.label("Sample Frequency (Hz):");
                    if ui
                        .add(egui::Slider::new(&mut sample_frequency, 1..=1000))
                        .changed()
                    {
                        sim.dt = 1.0 / sample_frequency as f32;
                        (estimates, rmse) = simulate_filter(&mut sim, &mut filter);
                    };

                    ui.label("Launch Angle (deg):");
                    if ui
                        .add(egui::Slider::new(&mut sim.launch_angle, 0.0..=45.0))
                        .changed()
                    {
                        (estimates, rmse) = simulate_filter(&mut sim, &mut filter);
                    };

                    ui.label("Sensor Noise:");
                    if ui
                        .add(egui::Slider::new(&mut sim.sensor_noise, 0.0..=1.0))
                        .changed()
                    {
                        (estimates, rmse) = simulate_filter(&mut sim, &mut filter);
                    };
                });
        });

        // --- Draw Trajectory ---
        for w in sim.history.windows(2) {
            let p1 = Point3::from(w[0].position);
            let p2 = Point3::from(w[1].position);
            window.draw_line(&p1, &p2, &Point3::new(1.0, 1.0, 1.0));
        }

        for w in estimates.windows(2) {
            let p1 = Point3::from(w[0].position);
            let p2 = Point3::from(w[1].position);
            window.draw_line(&p1, &p2, &Point3::new(1.0, 0.0, 0.0));
        }

        // --- Draw Ground Grid (Z=0 Plane) ---
        for i in -10..=10 {
            // Lines parallel to X-axis
            let start = Point3::new(-100.0, i as f32 * 10.0, 0.0);
            let end = Point3::new(100.0, i as f32 * 10.0, 0.0);
            window.draw_line(&start, &end, &Point3::new(0.3, 0.3, 0.3));

            // Lines parallel to Y-axis
            let start = Point3::new(i as f32 * 10.0, -100.0, 0.0);
            let end = Point3::new(i as f32 * 10.0, 100.0, 0.0);
            window.draw_line(&start, &end, &Point3::new(0.3, 0.3, 0.3));
        }
    }
}
