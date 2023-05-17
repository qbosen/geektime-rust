use image::ImageOutputFormat;

use crate::pb::Spec;
mod photon;
pub use photon::Photon;

/// 图片处理引擎trait
pub trait Engine {
    /// 根据spec配置engine
    fn apply(&mut self, specs: &[Spec]);
    /// 从engine生成图片
    fn generate(self, format: ImageOutputFormat) -> Vec<u8>;
}

/// 每个spec对应到图片的一种transform
pub trait SpecTransform<T> {
    fn transform(&mut self, op: T);
}
