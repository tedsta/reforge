#[derive(Clone, RustcEncodable, RustcDecodable)]
pub enum DamageVisualKind {
    Fire,
    Smoke,
}

#[derive(Clone, RustcEncodable, RustcDecodable)]
pub struct DamageVisual {
    pub x: f64,
    pub y: f64,
    pub kind: DamageVisualKind,
}