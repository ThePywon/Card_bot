use std::fmt::Display;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct DamageType {
  pub name: &'static str,
  pub icon: &'static str,
  pub attributes: i32
}

impl Display for DamageType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} {}", self.name, self.icon)
  }
}

pub const PHYSICAL: DamageType = DamageType { name: "physical", icon: ":crossed_swords:", attributes: 1 };
pub const POISON: DamageType = DamageType { name: "poison", icon: ":test_tube:", attributes: 8 };
pub const ACID: DamageType = DamageType { name: "acid", icon: ":biohazard:", attributes: 5 };
pub const VAMPIRIC: DamageType = DamageType { name: "vampiric", icon: ":drop_of_blood:", attributes: 3 };
#[allow(dead_code)]
pub const FIRE: DamageType = DamageType { name: "fire", icon: ":fire:", attributes: 5 };

pub const DAMAGE_ON_HIT: i32 = 0b0001;
pub const IS_VAMPIRIC: i32 = 0b0010;
#[allow(dead_code)]
pub const IGNORE_SHIELD: i32 = 0b0100;
#[allow(dead_code)]
pub const STACKS: i32 = 0b1000;

impl DamageType {
  pub fn damage_on_hit(&self) -> bool {
    self.attributes & DAMAGE_ON_HIT != 0
  }

  pub fn is_vampiric(&self) -> bool {
    self.attributes & IS_VAMPIRIC != 0
  }

  #[allow(dead_code)]
  pub fn ignore_shield(&self) -> bool {
    self.attributes & IGNORE_SHIELD != 0
  }

  #[allow(dead_code)]
  pub fn stacks(&self) -> bool {
    self.attributes & STACKS != 0
  }
}

// stacks  ignore_shield  vamp  hit_dmg
//   0           0          0      0   

