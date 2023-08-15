use std::fmt::Display;
use super::DamageType;

#[derive(Debug, Clone, Copy)]
pub struct Attack {
  pub name: &'static str,
  pub cost: u8,
  pub damage: u8,
  pub t: DamageType,
  pub required_traits: u8,
  pub forbidden_traits: u8,
  pub prob: f32
}

impl Display for Attack {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "  \\> `{}` {} 🔵  {} {}\n", self.name, self.cost, self.damage, self.t.icon)
  }
}
