use rand::rngs::ThreadRng;
use rand::Rng;

use crate::vector_3d::Vector3D;

#[derive(Clone, Debug)]
pub struct Physics {
    pub shape: Vec<[f64; 3]>,
    pub position: Vector3D,
    pub velocity: Vector3D,
    pub acceleration: Vector3D,
    pub spin_velocity: Vector3D,
    pub spin_acceleration: Vector3D,
    pub mass: f64,
    pub scale: f64,
    pub g_const: f64,
}

impl Physics {
    pub fn new(shape: Vec<[f64; 3]>) -> Physics {
        let position: Vector3D = Vector3D::new(0.0, 0.0, 0.0);
        let velocity: Vector3D = Vector3D::new(0.0, 0.0, 0.0);
        let acceleration: Vector3D = Vector3D::new(0.0, 0.0, 0.0);
        let spin_velocity: Vector3D = Vector3D::new(0.0, 0.0, 0.0);
        let spin_acceleration: Vector3D = Vector3D::new(0.0, 0.0, 0.0);
        let mass: f64 = 1.0;
        let scale: f64 = 1.0;
        let g_const: f64 = 0.0001;

        Physics {
            shape,
            position,
            velocity,
            acceleration,
            spin_velocity,
            spin_acceleration,
            mass,
            scale,
            g_const,
        }
    }

    fn rotate_x(&self, xyz_point: [f64; 3], theta: f64) -> [f64; 3] {
        let cs: f64 = theta.cos();
        let sn: f64 = theta.sin();
        let x: f64 = xyz_point[0];
        let y: f64 = cs * xyz_point[1] - sn * xyz_point[2];
        let z: f64 = sn * xyz_point[1] + cs * xyz_point[2];
        let xyz_point: [f64; 3] = [x, y, z];
        xyz_point
    }

    fn rotate_y(&self, xyz_point: [f64; 3], theta: f64) -> [f64; 3] {
        let cs: f64 = theta.cos();
        let sn: f64 = theta.sin();
        let x: f64 = cs * xyz_point[0] + sn * xyz_point[2];
        let y: f64 = xyz_point[1];
        let z: f64 = -sn * xyz_point[0] + cs * xyz_point[2];
        let xyz_point: [f64; 3] = [x, y, z];
        xyz_point
    }

    fn rotate_z(&self, xyz_point: [f64; 3], theta: f64) -> [f64; 3] {
        let cs: f64 = theta.cos();
        let sn: f64 = theta.sin();
        let x: f64 = cs * xyz_point[0] - sn * xyz_point[1];
        let y: f64 = sn * xyz_point[0] + cs * xyz_point[1];
        let z: f64 = xyz_point[2];
        let xyz_point: [f64; 3] = [x, y, z];
        xyz_point
    }

    fn constrain(&self, value: f64, min_value: f64, max_value: f64) -> f64 {
        f64::max(min_value, value).min(max_value)
    }

    fn calculate_position(&mut self, timestep: f64) {
        let timestep_velocity = self.velocity.multiply(timestep);
        let timestep_acceleration = self.acceleration.multiply(timestep);
        self.position = self.position.add_vector(timestep_velocity);
        self.velocity = self.velocity.add_vector(timestep_acceleration);
    }

    fn calculate_spin(&mut self, timestep: f64) {
        let timestep_spin_acc = self.spin_acceleration.multiply(timestep);
        self.spin_velocity = self.spin_velocity.add_vector(timestep_spin_acc);
        let x_rotation: f64 = self.spin_velocity.x * timestep;
        let y_rotation: f64 = self.spin_velocity.y * timestep;
        let z_rotation: f64 = self.spin_velocity.z * timestep;

        let shape: &mut Vec<[f64; 3]> = &mut Vec::new();
        for point in &self.shape {
            let mut point: [f64; 3] = *point;
            point = self.rotate_x(point, x_rotation);
            point = self.rotate_y(point, y_rotation);
            point = self.rotate_z(point, z_rotation);
            shape.push(point);
        }
        self.shape = shape.to_vec();
    }

    fn get_random_direction(&self) -> Vector3D {
        let mut rng: ThreadRng = rand::thread_rng();
        let x_rnd: f64 = rng.gen_range(-1.0..1.0);
        let y_rnd: f64 = rng.gen_range(-1.0..1.0);

        let direction = Vector3D::new(x_rnd, y_rnd, 0.0);
        direction
    }

    pub fn set_position(&mut self, x: f64, y: f64, z: f64) {
        self.position = Vector3D::new(x, y, z);
    }

    pub fn set_velocity(&mut self, x: f64, y: f64, z: f64) {
        self.velocity = Vector3D::new(x, y, z);
    }

    pub fn set_spin_velocity(&mut self, x: f64, y: f64, z: f64) {
        self.spin_velocity = Vector3D::new(x, y, z);
    }

    pub fn set_acceleration(&mut self, x: f64, y: f64, z: f64) {
        self.acceleration = Vector3D::new(x, y, z);
    }

    pub fn set_mass(&mut self, mass: f64) {
        self.mass = mass;
    }

    pub fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
    }

    pub fn correct_shift_collision(
        &mut self,
        target: &mut Physics,
        timestep: f64,
        direction: Vector3D,
        edge_distance: f64,
    ) {
        let edge: f64 = edge_distance + timestep;
        let mut direction: Vector3D = direction;

        if direction.get_length_squared() == 0.0 {
            direction = self.get_random_direction();
        }

        let self_edge_vec = direction.multiply(-edge);
        let target_edge_vec = direction.multiply(edge);

        let self_shifted: Vector3D = self.position.add_vector(self_edge_vec);
        let target_shifted: Vector3D = target.position.add_vector(target_edge_vec);

        self.position = self_shifted;
        target.position = target_shifted;
    }

    pub fn calculate_collision_velocities(&mut self, target: &mut Physics, direction: Vector3D) {
        let v1i: f64 = self.velocity.dot_product(direction);
        let v2i: f64 = target.velocity.dot_product(direction);
        let v1i_vec: Vector3D = direction.multiply(v1i);
        let v2i_vec: Vector3D = direction.multiply(v2i);
        let v1p: Vector3D = self.velocity.subtract_vector(v1i_vec);
        let v2p: Vector3D = target.velocity.subtract_vector(v2i_vec);

        let m1: f64 = self.mass;
        let m2: f64 = target.mass;

        let v1f: f64 = ((v1i * (m1 - m2)) + 2.0 * (m2 * v2i)) / (m1 + m2);
        let v2f: f64 = ((v2i * (m2 - m1)) + 2.0 * (m1 * v1i)) / (m1 + m2);

        let v1f_vec = direction.multiply(v1f);
        let v2f_vec = direction.multiply(v2f);

        let v1: Vector3D = v1p.add_vector(v1f_vec);
        let v2: Vector3D = v2p.add_vector(v2f_vec);

        self.velocity = v1;
        target.velocity = v2;
    }

    pub fn apply_forces(&mut self, target: &mut Physics, timestep: f64) {
        // Target-To-Self Distance
        let tts_distance: Vector3D = target.position.subtract_vector(self.position);

        self.apply_attraction(target, tts_distance);
        self.apply_collision(target, tts_distance, timestep);
    }

    pub fn apply_attraction(&mut self, target: &mut Physics, tts_distance: Vector3D) {
        let distance: f64 = tts_distance.get_length();

        if distance > 0.0 {
            let mut force: Vector3D = target.position.subtract_vector(self.position);
            let strength: f64 = self.g_const * ((self.mass * target.mass) / distance);
            force = tts_distance.set_magnitude(strength);
            force = force.divide(self.mass);
            self.acceleration = self.acceleration.add_vector(force);
            self.spin_acceleration = self.spin_acceleration.add_vector(force);
        }
    }

    pub fn apply_collision(&mut self, target: &mut Physics, tts_distance: Vector3D, timestep: f64) {
        let self_radius: f64 = self.scale + self.position.get_length() * timestep;
        let target_radius: f64 = target.scale + target.position.get_length() * timestep;

        let total_radius: f64 = self_radius + target_radius;
        let edge_distance: f64 = tts_distance.get_length() - total_radius;

        if edge_distance <= 0.0 {
            // Self-To-Target Distance
            let stt_distance: Vector3D = tts_distance.multiply(-1.0);
            let stt_direction: Vector3D = stt_distance.normalize();

            self.calculate_collision_velocities(target, stt_direction);
            self.correct_shift_collision(target, timestep, stt_direction, edge_distance);
        }
    }

    pub fn update(&mut self, timestep: f64) {
        self.calculate_position(timestep);
        self.calculate_spin(timestep);
        self.acceleration = self.acceleration.multiply(0.0);
        self.spin_acceleration = self.spin_acceleration.multiply(0.0);
    }
}
