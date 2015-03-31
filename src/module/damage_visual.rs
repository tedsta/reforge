#[derive(Clone, Copy, RustcEncodable, RustcDecodable)]
pub enum DamageVisualKind {
    Fire,
    Smoke,
}

#[derive(Clone, Copy, RustcEncodable, RustcDecodable)]
pub struct DamageVisual {
    pub x: f64,
    pub y: f64,
    pub kind: DamageVisualKind,
}