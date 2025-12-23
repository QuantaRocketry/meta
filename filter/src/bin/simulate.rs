use kiss3d::{camera::ArcBall, light::Light};
use kiss3d::text::Font;
use kiss3d::window::Window;
use nalgebra::*;
use smol::{Unblock, io, net, prelude::*};

// --- 1. Define the Rocket State ---
#[derive(Debug, Clone)]
struct RocketState {
    time: f32,
    position: Vector3<f32>,
    velocity: Vector3<f32>,
    acceleration: Vector3<f32>,
    orientation: UnitQuaternion<f32>,
    angular_velocity: Vector3<f32>,
}

impl RocketState {
    fn new() -> Self {
        Self {
            time: 0.0,
            position: Vector3::new(0.0, 0.0, 0.0),
            velocity: Vector3::new(0.0, 0.0, 0.0),
            acceleration: Vector3::new(0.0, 0.0, 0.0),
            orientation: UnitQuaternion::from_euler_angles(-0.10, 0.0, 0.0),
            angular_velocity: Vector3::new(0.0, 0.0, 0.0),
        }
    }
}

// --- 2. The Simulation Engine ---
struct RocketSimulation {
    history: Vec<RocketState>,
    mass: f32,
    dt: f32,
}

impl RocketSimulation {
    fn new(mass: f32, dt: f32) -> Self {
        let initial_state = RocketState::new();
        Self {
            history: vec![initial_state],
            mass,
            dt,
        }
    }

    fn run(&mut self, duration: f32) {
        let steps = (duration / self.dt) as usize;

        for _ in 0..steps {
            let prev = self.history.last().unwrap();
            let mut next = prev.clone();

            // Forces
            let gravity = Vector3::new(0.0, 0.0, -9.81) * self.mass;

            let thrust_force = if prev.time < 1.0 {
                25.0 * self.mass
            } else {
                0.0
            };

            let thrust_world = prev.orientation * Vector3::new(0.0, 0.0, thrust_force);

            // Torque (tweak: reduced torque slightly for better visualization)
            let torque = if prev.time > 1.0 && prev.time < 2.0 {
                Vector3::new(0.1, 0.0, 0.0)
            } else {
                Vector3::new(0.0, 0.0, 0.0)
            };

            // Integration
            let total_force = gravity + thrust_world;
            next.acceleration = total_force / self.mass;
            next.velocity += next.acceleration * self.dt;
            next.position += next.velocity * self.dt;

            let angular_accel = torque;
            next.angular_velocity += angular_accel * self.dt;

            let rotation_delta = UnitQuaternion::from_euler_angles(
                next.angular_velocity.x * self.dt,
                next.angular_velocity.y * self.dt,
                next.angular_velocity.z * self.dt,
            );
            next.orientation = rotation_delta * prev.orientation;

            next.time += self.dt;
            self.history.push(next.clone());

            if next.position.z < 0.0 {
                break;
            }
        }
    }
}

fn main() -> io::Result<()> {
    smol::block_on(async {
        println!("Calculating physics...");
        let mut sim = RocketSimulation::new(1000.0, 0.01);
        sim.run(20.0);
        println!("Simulation done. Steps: {}", sim.history.len());

        let mut window = Window::new("Rocket Flight 3D");
        window.set_light(Light::StickToCamera);

        // --- SETUP CAMERA ---
        let eye = Point3::new(-40.0, -40.0, 20.0);
        // At: Look at the origin (0,0,0)
        let at = Point3::origin();
        let mut camera = ArcBall::new(eye, at);
        camera.set_up_axis(*Vector3::z_axis());

        // Create the rocket visual (Red Cone)
        let mut rocket_gfx = window.add_cone(1.0, 4.0);
        rocket_gfx.set_color(1.0, 0.2, 0.2);

        let mut frame_idx = 0;
        let playback_speed = 3;

        while window.render_with_camera(&mut camera).await {
            // --- Draw Trajectory ---
            for i in 0..sim.history.len() - 1 {
                let p1 = Point3::from(sim.history[i].position);
                let p2 = Point3::from(sim.history[i + 1].position);
                window.draw_line(&p1, &p2, &Point3::new(1.0, 1.0, 1.0));
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

            // Draw small axes at origin for reference (R=X, G=Y, B=Z)
            // window.draw_line(
            //     &Point3::origin(),
            //     &Point3::new(5.0, 0.0, 0.0),
            //     &Point3::new(1.0, 0.0, 0.0),
            // );
            // window.draw_line(
            //     &Point3::origin(),
            //     &Point3::new(0.0, 5.0, 0.0),
            //     &Point3::new(0.0, 1.0, 0.0),
            // );
            // window.draw_line(
            //     &Point3::origin(),
            //     &Point3::new(0.0, 0.0, 5.0),
            //     &Point3::new(0.0, 0.0, 1.0),
            // );

            // --- Update Rocket Animation ---
            if frame_idx < sim.history.len() {
                let state = &sim.history[frame_idx];

                let t = Translation3::from(state.position);
                rocket_gfx.set_local_translation(t);

                // Rotate default cone (Y-up) to match Physics (Z-up)
                let correction = UnitQuaternion::from_axis_angle(
                    &Vector3::x_axis(),
                    -std::f32::consts::FRAC_PI_2,
                );
                rocket_gfx.set_local_rotation(state.orientation * correction);

                window.draw_text(
                    &format!(
                        "Time: {:.2}s\nAlt: {:.1}m\nVel: {:.1} m/s",
                        state.time,
                        state.position.z,
                        state.velocity.norm()
                    ),
                    &Point2::new(10.0, 10.0),
                    20.0,
                    &Font::default(),
                    &Point3::new(1.0, 1.0, 1.0),
                );

                frame_idx += playback_speed;
            } else {
                frame_idx = 0;
            }
        }
        Ok(())
    })
}
