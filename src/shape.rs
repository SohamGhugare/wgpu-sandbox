use std::f32::consts::PI;

#[derive(Clone, clap::ValueEnum)]
pub enum Shape {
    Triangle,
    Square,
    Circle,
}

pub struct ShapeConfig {
    pub shape: Shape,
    pub color: [f32; 4],
    pub size: f32,
    pub position: [f32; 2],
    pub axis: bool,
    pub axis_grid: bool,
    pub axis_arm_len: f32,
}

impl ShapeConfig {
    pub fn vertices(&self) -> Vec<[f32; 2]> {
        let s = self.size;
        match self.shape {
            Shape::Triangle => vec![
                [-s, -s],
                [0.0, s],
                [s, -s],
            ],
            Shape::Square => vec![
                [-s, -s], [s, -s], [s,  s],
                [-s, -s], [s,  s], [-s, s],
            ],
            Shape::Circle => {
                let segments = 64u32;
                let mut v = Vec::with_capacity((segments * 3) as usize);
                for i in 0..segments {
                    let a1 = 2.0 * PI * i as f32 / segments as f32;
                    let a2 = 2.0 * PI * (i + 1) as f32 / segments as f32;
                    v.push([0.0_f32, 0.0]);
                    v.push([a1.cos() * s, a1.sin() * s]);
                    v.push([a2.cos() * s, a2.sin() * s]);
                }
                v
            }
        }
    }
}
