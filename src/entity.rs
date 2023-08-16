use std::{fmt::Display, collections::HashMap};
use rand::random;
use uuid::Uuid;

pub mod state;
use state::ALIVE;
pub mod attributes;
pub mod dmg_type;
use dmg_type::{DamageType, PHYSICAL, POISON, ACID, VAMPIRIC};
pub mod dmg_resistance;
use dmg_resistance::{DamageResistance, DamageResistance::NEUTRAL};
pub mod ability;
pub use ability::{Ability, AbilityTrigger, AbilityEffect};
pub mod attack;
pub use attack::Attack;


#[derive(Debug, Clone)]
pub struct Entity {
  pub id: Uuid,
  pub name: &'static str,
  pub description: &'static str,
  pub max_health: u8,
  pub current_health: u8,
  pub abilities: Vec<&'static Ability>,
  pub attacks: Vec<&'static Attack>,
  pub state: u8,
  pub stacks: Vec<(DamageType, u8)>,
  pub attributes: u8,
  pub damage_resistance: HashMap<DamageType, DamageResistance>
}

impl Display for Entity {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut ability_descs = String::new();
    for ability in self.abilities.iter() {
      ability_descs += &ability.to_string();
    }
    let mut attack_descs = String::new();
    for attack in self.attacks.iter() {
      attack_descs += &attack.to_string();
    }
    write!(f, "**{}**  {}/{} ❤️ {}\n{}{}", self.name,
      self.current_health, self.max_health, if self.is_alive() {' '} else {'💀'}, ability_descs, attack_descs)
  }
}

pub struct EntityBuilder {
  pub name: &'static str,
  pub description: &'static str,
  pub base_health: (u8, u8),
  pub base_properties: (u8, Vec<(DamageType, DamageResistance)>),
  pub variant_properties: Vec<(u8, Vec<(DamageType, DamageResistance)>, f32)>,
  pub abilities: Vec<&'static Ability>,
  pub attacks: Vec<&'static Attack>
}

impl Entity {
  #[allow(dead_code)]
  pub fn take_damage(&mut self, amt: u8, t: dmg_type::DamageType) -> (String, u8) {
    if self.state & ALIVE == 0 {
      return (String::new(), 0);
    }

    let mut result = String::new();

    println!("\"{}\" is being attacked for {} {} damage.", self.name, amt, t);
    result += &format!("**{}** is being attacked for {} {} damage.\n", self.name, amt, t);

    let multiplier: f64;
    match self.damage_resistance.get(&t).unwrap() {
      DamageResistance::WEAKNESS => {
        multiplier = 1.5;
        println!("\"{}\" is weak to {} damage!", self.name, t);
        result += &format!("**{}** is weak to {} damage!\n", self.name, t);
      },
      DamageResistance::NEUTRAL => multiplier = 1.0,
      DamageResistance::RESISTANCE => {
        multiplier = 0.5;
        println!("\"{}\" is resistant to {} damage!", self.name, t);
        result += &format!("**{}** is resistant to {} damage!\n", self.name, t);
      },
      DamageResistance::IMMUNITY => {
        multiplier = 0.0;
        println!("\"{}\" is immune to {} damage!", self.name, t);
        result += &format!("**{}** is immune to {} damage!\n", self.name, t);
      }
    }

    let actual_amount = (amt as f64 * multiplier) as u8;
    println!("\"{}\" took {} {} damage.", self.name, actual_amount, t);
    result += &format!("**{}** took {} {} damage.\n", self.name, actual_amount, t);
    self.current_health -= actual_amount;

    (result, actual_amount)
  }

  pub fn heal(&mut self, amt: u8) -> u8 {
    if self.state & ALIVE == 0 ||
      self.current_health == self.max_health {
      return 0;
    }

    let old_health = self.current_health;

    self.current_health += amt;
    if self.current_health >= self.max_health {
      self.current_health = self.max_health;
    }

    self.current_health - old_health
  }

  pub fn get_attack(&self, attack_name: &str) -> Option<Attack> {
    for attack in self.attacks.iter() {
      if attack.name == attack_name {
        return Some((*attack).clone());
      }
    }

    None
  }

  #[allow(dead_code)]
  pub fn check_for_trigger(&mut self, trigger: AbilityTrigger, ability_queue: &mut Vec<(Ability, u8, u8)>, source: u8, target: u8) -> String {
    let mut result = String::new();
    for ability in self.abilities.iter() {
      if trigger == ability.trigger {
        println!("\"{}\" has been triggered! (#{})", ability.name, source);
        result += &format!("`{}` has been triggered! (#{})\n", ability.name, source);

        ability_queue.push(((*ability).clone(), source, target));
      }
    }
    result
  }

  pub fn died(&mut self) -> bool {
    if self.current_health > 0 || self.state & ALIVE == 0 {
      return false;
    }

    self.current_health = 0;
    self.state ^= ALIVE;
    true
  }

  pub fn is_alive(&self) -> bool {
    self.state & ALIVE != 0
  }
}

impl EntityBuilder {
  pub fn new(name: &'static str, description: &'static str,
    base_health: (u8, u8), base_properties: (u8, Vec<(DamageType, DamageResistance)>),
    variant_properties: Vec<(u8, Vec<(DamageType, DamageResistance)>, f32)>,
    abilities: Vec<&'static Ability>, attacks: Vec<&'static Attack>) -> Self {
      EntityBuilder { name, description, base_health,
        base_properties, variant_properties, abilities, attacks }
  }

  fn get_base_health(&self) -> u8 {
    (random::<f32>() * (self.base_health.1 - self.base_health.0 + 1) as f32) as u8 + self.base_health.0
  }

  fn get_properties(&self) -> (u8, HashMap<DamageType, DamageResistance>) {
    let mut traits = self.base_properties.0;
    let mut resistances: HashMap<DamageType, DamageResistance> = HashMap::from([
      (PHYSICAL, NEUTRAL),
      (POISON, NEUTRAL),
      (ACID, NEUTRAL),
      (VAMPIRIC, NEUTRAL)
    ]);

    for (k, v) in self.base_properties.1.iter() {
      resistances.insert(*k, *v);
    }

    for (t, r, prob) in self.variant_properties.iter() {
      if random::<f32>() <= *prob {
        traits |= *t;
        for (k, v) in r.iter() {
          resistances.insert(*k, *v);
        }
      }
    }

    (traits, resistances)
  }

  fn get_attacks(&self, traits: u8) -> Vec<&'static Attack> {
    let mut result = vec![];

    for attack in self.attacks.iter() {
      let r = attack.required_traits;
      let f = attack.forbidden_traits;
      if r & traits == r &&
        f & traits == 0 &&
        random::<f32>() <= attack.prob {
          result.push(*attack);
        }
    }

    result
  }

  fn get_abilities(&self, traits: u8) -> Vec<&'static Ability> {
    let mut result = vec![];

    for ability in self.abilities.iter() {
      let r = ability.required_traits;
      let f = ability.forbidden_traits;
      if r & traits == r &&
        f & traits == 0 &&
        random::<f32>() <= ability.probability {
          result.push(*ability);
        }
    }

    result
  }

  #[allow(dead_code)]
  pub fn build(&self) -> Entity {
    let max_health = self.get_base_health();
    let properties = self.get_properties();
    Entity { id: Uuid::new_v4(), name: self.name, description: self.description, max_health, current_health: max_health,
      abilities: self.get_abilities(properties.0), attacks: self.get_attacks(properties.0),
      state: ALIVE, stacks: Vec::new(), attributes: properties.0, damage_resistance: properties.1 }
  }
}
