#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum DamageVisualKind {
    Fire,
    Smoke,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct DamageVisual {
    pub x: f64,
    pub y: f64,
    pub kind: DamageVisualKind,
}