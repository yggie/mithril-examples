use graphics;
use std::rc::Rc;

pub struct Object<'a> {
    asset: Rc<graphics::Asset<'a>>,
    translation: [f32; 3],
    scale: f32,
}


impl<'a> Object<'a> {
    pub fn new(asset: Rc<graphics::Asset<'a>>) -> Object<'a> {
        Object{
            asset: asset,
            translation: [0.0; 3],
            scale: 1.0,
        }
    }


    #[inline]
    pub fn asset(&self) -> &graphics::Asset {
        &*self.asset
    }


    #[inline]
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }


    #[inline]
    pub fn set_translation(&mut self, x: f32, y: f32, z: f32) {
        self.translation = [x, y, z];
    }


    #[inline]
    pub fn model_matrix(&self) -> [f32; 16] {
        [
            self.scale,        0.0,        0.0, self.translation[0],
                   0.0, self.scale,        0.0, self.translation[1],
                   0.0,        0.0, self.scale, self.translation[2],
                   0.0,        0.0,        0.0,                 1.0,
        ]
    }
}
