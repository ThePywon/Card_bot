use crate::entity::{Entity, AbilityTrigger, dmg_type::DamageType, AbilityEffect};
use crate::entity::dmg_resistance::DamageResistance::{WEAKNESS, RESISTANCE, IMMUNITY};
use crate::entity::ability::{trigger_target::{ITSELF, ALLY, OPPONENT},
  AbilityTriggerType::{AnyDamage, Damage}, AbilityEffectTarget};

#[allow(dead_code, non_snake_case)]
pub struct Scene {
  pub teams: Vec<(&'static str, Vec<Entity>)>
}

impl Scene {
  pub fn new() -> Self {
    Scene { teams: Vec::new() }
  }
  
  #[allow(non_snake_case)]
  pub fn register(&mut self, team: &'static str, entity: &Entity) -> u8 {
    for (index, (name, entities)) in self.teams.iter_mut().enumerate() {
      if *name == team {
        let entity_index = entities.len() as u8;
        entities.push(entity.clone());
        let result = entity_index << 2 | index as u8;
        return result;
      }
    }

    let team_index = self.teams.len() as u8;

    if team_index > 4 {
      panic!("Team non available for registration.");
    }

    self.teams.push((team, vec![entity.clone()]));
    team_index
  }

  #[allow(dead_code)]
  pub fn attack(&mut self, attack_name: &str, attacker: u8, target: u8) -> String {
    // Check if entities exist
    if let None = self.get_entity_from_id(attacker) {
      let result = format!("Could not find attacking entity with id #{}", attacker);
      println!("{}", result);
      return result;
    }
    if let None = self.get_mut_entity_from_id(target) {
      let result = format!("Could not find target entity with id #{}", target);
      println!("{}", result);
      return result;
    }

    let attack = self.get_mut_entity_from_id(attacker).unwrap().get_attack(attack_name);

    // Check if attack exists
    if let None = attack { 
      let name = self.get_entity_from_id(attacker).unwrap().name;
      println!("\"{}\"#{} does not know the \"{}\" attack", name, attacker, attack_name);
      return format!("**{}**#{} does not know the `{}` attack", name, attacker, attack_name);
    }

    // Check entities are alive
    if !self.get_entity_from_id(attacker).unwrap().is_alive() {
      let name = self.get_entity_from_id(attacker).unwrap().name;
      println!("\"{}\"#{} tried using \"{}\" but was unconscious", name, attacker, attack_name);
      return format!("**{}**#{} tried using `{}` but was unconscious!", name, attacker, attack_name);
    }

    if !self.get_entity_from_id(target).unwrap().is_alive() {
      let name = self.get_entity_from_id(target).unwrap().name;
      println!("\"{}\"#{} was targetted by the \"{}\" attack but is already unconscious", name, target, attack_name);
      return format!("**{}**#{} was targetted by the `{}` attack but is already unconscious!", name, target, attack_name);
    }

    // Variables
    let mut result = String::new();

    // Print attack messages
    let mut e = self.get_mut_entity_from_id(attacker).unwrap();
    print!("\"{}\"#{} is using the \"{}\" attack", e.name, attacker, attack_name);
    result += &format!("**{}**#{} is using the `{}` attack", e.name, attacker, attack_name);
    e = self.get_mut_entity_from_id(target).unwrap();
    println!(" on \"{}\"#{}", e.name, target);
    result += &format!(" on **{}**#{}\n", e.name, target);

    if attack.unwrap().t.damage_on_hit() {
      result += &self.deal_damage(attack.unwrap().t, attack.unwrap().damage, attacker, target);
    }

    let mut ability_queue: Vec<(String, AbilityEffect, u8, u8)> = Vec::new();

    // Check for triggers
    for (team_index, (_, entities)) in self.teams.iter().enumerate() {
      for (entity_index, entity) in entities.iter().enumerate() {
        let (lhs, rhs): (u8, u8);

        if team_index == (attacker & 3) as usize {
          if entity_index == (attacker >> 2) as usize {
            lhs = ITSELF;
          }
          else {
            lhs = ALLY;
          }
        }
        else {
          lhs = OPPONENT;
        }

        if team_index == (target & 3) as usize {
          if entity_index == (target >> 2) as usize {
            rhs = ITSELF;
          }
          else {
            rhs = ALLY;
          }
        }
        else {
          rhs = OPPONENT;
        }

        let e = entity.clone();
        if let Some((name, effect)) = e.check_for_trigger(AbilityTrigger { t: Damage(attack.unwrap().t), source: lhs, target: rhs }) {
          let itself = ((entity_index << 2) | team_index) as u8;
          
          match effect.target {
            AbilityEffectTarget::Itself => ability_queue.push((String::from(name), effect, itself, itself)),
            AbilityEffectTarget::TriggerTarget => ability_queue.push((String::from(name), effect.clone(), itself, target)),
            _ => {}
          }
        }
        if let Some((name, effect)) = e.check_for_trigger(AbilityTrigger { t: AnyDamage, source: lhs, target: rhs }) {
          let itself = ((entity_index << 2) | team_index) as u8;
          match effect.target {
            AbilityEffectTarget::Itself => ability_queue.push((String::from(name), effect.clone(), itself, itself)),
            AbilityEffectTarget::TriggerTarget => ability_queue.push((String::from(name), effect.clone(), itself, target)),
            _ => {}
          }
        }
      }
    }

    // Resolve ability queue
    for (name, effect, source, target) in ability_queue.iter() {
      println!("Ability \"{}\" triggered!", name);
      result += &format!("Ability `{}` triggered!", name);
      self.deal_damage(effect.t, effect.damage, *source, *target);
    }

    // Resolve any deaths that occured after the queued attacks
    for (_, team) in self.teams.iter_mut() {
      for entity in team.iter_mut() {
        if entity.died() {
          println!("\"{}\" has fainted!", entity.name);
          result += &format!("**{}** has fainted!", entity.name);
        }
      }
    }

    result
  }

  pub fn deal_damage(&mut self, dmg_type: DamageType, dmg_amt: u8, attacker_id: u8, target_id: u8) -> String {
    // Variables
    let mut result = String::new();

    // Deal damage
    let mut entity = self.get_mut_entity_from_id(target_id).unwrap();
    // Print damage messages
    println!("\"{}\" is being attacked for {} {} damage.", entity.name, dmg_amt, dmg_type);
    result += &format!("**{}** is being attacked for {} {} damage.\n", entity.name, dmg_amt, dmg_type);
    if let Some(resistance) = entity.damage_resistance.get(&dmg_type) {
      match resistance {
        WEAKNESS => {
          println!("\"{}\" is weak to {} damage!", entity.name, dmg_type);
          result += &format!("**{}** is weak to {} damage!\n", entity.name, dmg_type);
        },
        RESISTANCE => {
          println!("\"{}\" is resistant to {} damage!", entity.name, dmg_type);
          result += &format!("**{}** is resistant to {} damage!\n", entity.name, dmg_type);
        },
        IMMUNITY => {
          println!("\"{}\" is immune to {} damage!", entity.name, dmg_type);
          result += &format!("**{}** is immune to {} damage!\n", entity.name, dmg_type);
        }
        _ => {}
      }
    }

    // Calculate and deal damage
    let damage = dmg_amt as f32 * entity.get_dmg_multiplier(dmg_type);
    entity.take_damage(damage as u8);

    // Print more damage messages
    println!("\"{}\" suffered {} {} damage.", entity.name, damage, dmg_type);
    result += &format!("**{}** suffered {} {} damage.\n", entity.name, damage, dmg_type);

    // Heal vampiric damage and print message
    if dmg_type.is_vampiric() {
      entity = self.get_mut_entity_from_id(attacker_id).unwrap();
      let healed_amt = entity.heal(damage as u8);

      println!("\"{}\" is being healed for {} ❤️ ", entity.name, damage);
      result += &format!("**{}** is being healed for {} :heart: \n", entity.name, damage);
      println!("\"{}\" got healed by {} ❤️ ", entity.name, healed_amt);
      result += &format!("**{}** got healed by {} :heart: \n", entity.name, healed_amt);
    }

    result
  }

  pub fn get_mut_entity_from_id(&mut self, id: u8) -> Option<&mut Entity> {
    if let Some((_, entities)) = self.teams.get_mut((id & 3) as usize) {
      let index = (id >> 2) as usize;

      if entities.len() > index {
        return Some(&mut entities[(id >> 2) as usize]);
      }
    }

    None
  }

  pub fn get_entity_from_id(&self, id: u8) -> Option<&Entity> {
    if let Some((_, entities)) = self.teams.get((id & 3) as usize) {
      let index = (id >> 2) as usize;

      if entities.len() > index {
        return Some(&entities[(id >> 2) as usize]);
      }
    }

    None
  }

  #[allow(dead_code)]
  pub fn describe_entity(&self, id: u8) -> Option<String> {
    if let Some(entity) = self.get_entity_from_id(id) {
      let result = format!("(#{})\n{}\n", id, entity);
      println!("{}", result);
      Some(result)
    }
    else {
      println!("No entity found with id: {}", id);
      None
    }
  }

  #[allow(dead_code)]
  pub fn describe_scene(&self) -> String {
    let mut result = String::new();

    for (team_index, (name, entities)) in self.teams.iter().enumerate() {
      result += &format!("Team \"{}\"\n", name);
      for (entity_index, entity) in entities.iter().enumerate() {
        let id = (entity_index << 2 | team_index) as u8;
        result += &format!("#{}\n{}\n", id, entity);
      }
    }

    result
  }

  #[allow(dead_code)]
  pub fn nuke(&mut self) {
    self.teams = Vec::new();
  }

  #[allow(dead_code)]
  pub fn is_empty(&self) -> bool {
    self.teams.len() == 0
  }
}
