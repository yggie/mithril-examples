extern crate mithril;

use std::f64;
use std::num::Float;
use self::mithril::math::{ Vector, Quaternion };

pub struct Camera {
    position: Vector,
    focus_point: Vector,
    up: Vector,
    field_of_view: f64,
    aspect_ratio: f64,
    far: f64,
    near: f64,
    anchor_point: Option<[f64; 2]>,
    control_point: [f64; 2],
}

impl Camera {
    pub fn new(position: Vector, focus_point: Vector, up: Vector) -> Camera {
        Camera{
            position: position,
            focus_point: focus_point,
            up: up.normalize(),
            field_of_view: (90.0 * f64::consts::PI / 180.0),
            aspect_ratio: 640.0/480.0,
            far: 100.0,
            near: 1.0,
            anchor_point: None,
            control_point: [0.0; 2],
        }
    }

    pub fn position(&self) -> Vector {
        self.position
    }

    pub fn focus_point(&self) -> Vector {
        self.focus_point
    }

    pub fn go_to(&mut self, position: Vector) {
        self.position = position;
    }

    pub fn update(&mut self) {
    }

    pub fn start_control(&mut self, x: f64, y: f64) {
        self.anchor_point = Some([x, y]);
        self.control_point[0] = x;
        self.control_point[1] = y;
    }

    pub fn set_control_point(&mut self, x: f64, y: f64) {
        self.control_point[0] = x;
        self.control_point[1] = y;
    }

    pub fn release_controls(&mut self) {
        self.anchor_point = None;
    }

    pub fn is_controlled(&self) -> bool {
        self.anchor_point != None
    }

    pub fn view_matrix(&self) -> [f32; 16] {
        let mut z_view = (self.position - self.focus_point).normalize();
        let mut x_view = self.up.cross(z_view).normalize();
        let mut y_view = z_view.cross(x_view).normalize();

        let x_trans = -self.position.dot(x_view);
        let y_trans = -self.position.dot(y_view);
        let z_trans = -self.position.dot(z_view);

        match self.anchor_point {
            Some(anchor_point) => {
                let diff = [
                    (self.control_point[1] - anchor_point[1]) as f32,
                    (anchor_point[0] - self.control_point[0]) as f32,
                ];
                let diff_sq = (diff[0] * diff[0] + diff[1] * diff[1]).sqrt();
                if diff_sq > 0.0001 {
                    let diff_length = diff_sq.sqrt();

                    let rot_axis = (x_view * diff[0] + y_view * diff[1]) / diff_length;
                    let rot_in_radians = diff_length * 2.0;

                    let u_quat = Quaternion::new(0.0, x_view[0], x_view[1], x_view[2]);
                    let v_quat = Quaternion::new(0.0, y_view[0], y_view[1], y_view[2]);
                    let w_quat = Quaternion::new(0.0, z_view[0], z_view[1], z_view[2]);
                    let rot_quat = Quaternion::new_from_rotation(rot_in_radians, rot_axis[0], rot_axis[1], rot_axis[2]);

                    let new_u_quat = rot_quat * u_quat * rot_quat.inverse();
                    let new_v_quat = rot_quat * v_quat * rot_quat.inverse();
                    let new_w_quat = rot_quat * w_quat * rot_quat.inverse();
                    x_view[0] = new_u_quat[1];
                    x_view[1] = new_u_quat[2];
                    x_view[2] = new_u_quat[3];

                    y_view[0] = new_v_quat[1];
                    y_view[1] = new_v_quat[2];
                    y_view[2] = new_v_quat[3];

                    z_view[0] = new_w_quat[1];
                    z_view[1] = new_w_quat[2];
                    z_view[2] = new_w_quat[3];
                }
            }

            None => {
                // do nothing
            }
        }

        [
            x_view[0], x_view[1], x_view[2], x_trans,
            y_view[0], y_view[1], y_view[2], y_trans,
            z_view[0], z_view[1], z_view[2], z_trans,
                  0.0,       0.0,       0.0,     1.0,
        ]
    }

    pub fn projection_matrix(&self) -> [f32; 16] {
        let m_11 = (1.0 / (self.field_of_view / 2.0).tan()) as f32;
        let m_22 = m_11 * (self.aspect_ratio as f32);
        let m_33 = -((self.far + self.near) / (self.far - self.near)) as f32;
        let m_43 = -((2.0 * self.far * self.near) / (self.far - self.near)) as f32;
        [
            m_11,  0.0,  0.0,  0.0,
             0.0, m_22,  0.0,  0.0,
             0.0,  0.0, m_33, m_43,
             0.0,  0.0, -1.0,  0.0,
        ]
    }
}
