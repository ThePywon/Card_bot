use crate::entity::{Entity, AbilityTrigger, Ability, ability::{AbilityTriggerType,
  trigger_target::{ITSELF, ALLY, OPPONENT}, AbilityEffectTarget}};

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

    // Get entities
    let mut e = self.get_mut_entity_from_id(attacker).unwrap();

    // Check if attack exists
    if let None = e.get_attack(attack_name) { 
      println!("\"{}\"#{} does not know the \"{}\" attack", e.name, attacker, attack_name);
      return format!("**{}**#{} does not know the `{}` attack", e.name, attacker, attack_name);
    }



    // Get attack
    let attack = e.get_attack(attack_name).unwrap().clone();

    // Check if target is alive
    if !e.is_alive() {
      println!("\"{}\"#{} tried using \"{}\" but was unconscious", e.name, attacker, attack.name);
      return format!("**{}**#{} tried using `{}` but was unconscious!", e.name, attacker, attack.name);
    }

    e = self.get_mut_entity_from_id(target).unwrap();
    if !e.is_alive() {
      println!("\"{}\"#{} was targetted by the \"{}\" attack but is already unconscious", e.name, target, attack.name);
      return format!("**{}**#{} was targetted by the `{}` attack but is already unconscious!", e.name, target, attack.name);
    }

    // Variables
    let mut result = String::new();
    let mut ability_queue: Vec<(Ability, u8, u8)> = Vec::new();
    let dmg_taken: u8;

    e = self.get_mut_entity_from_id(attacker).unwrap();
    println!("\"{}\"#{} is using the \"{}\" attack", e.name, attacker, attack.name);
    result += &format!("**{}**#{} is using the `{}` attack", e.name, attacker, attack.name);
    e = self.get_mut_entity_from_id(target).unwrap();
    println!(" on \"{}\"#{}", e.name, target);
    result += &format!(" on **{}**#{}\n", e.name, target);
    

    // Deal damage to target and check for triggered abilities
    if attack.t.damage_on_hit() {
      let (x, y) = e.take_damage(attack.damage, attack.t);
      result += &x;
      dmg_taken = y;

      // target triggers
      if attacker != target {
        let source: u8;
        if attacker & 3 == target & 3 {
          source = ALLY;
        }
        else {
          source = OPPONENT;
        }

        result += &e.check_for_trigger(AbilityTrigger { t: AbilityTriggerType::AnyDamage, source, target: ITSELF },
          &mut ability_queue, attacker, target);
        result += &e.check_for_trigger(AbilityTrigger { t: AbilityTriggerType::Damage(attack.t), source, target: ITSELF },
          &mut ability_queue, attacker, target);
      }

      // Attacker triggers
      e = self.get_mut_entity_from_id(attacker).unwrap();
      let trigger_target: u8;
      if attacker == target {
        trigger_target = ITSELF;
      }
      else if attacker & 3 == target & 3 {
        trigger_target = ALLY;
      }
      else {
        trigger_target = OPPONENT;
      }

      result += &e.check_for_trigger(AbilityTrigger {
        t: AbilityTriggerType::AnyDamage, source: ITSELF, target: trigger_target },
        &mut ability_queue, attacker, target );
      result += &e.check_for_trigger(AbilityTrigger {
        t: AbilityTriggerType::Damage(attack.t), source: ITSELF, target: trigger_target },
        &mut ability_queue, attacker, target);


      // Check for vamp healing
      if attack.t.is_vampiric() {
        let healed_amt = e.heal(dmg_taken);
        println!("\"{}\" is being healed for {} ❤️ ", e.name, dmg_taken);
        println!("\"{}\" got healed by {} ❤️ ", e.name, healed_amt);
        result += &format!("**{}** is being healed for {} ❤️ \n", e.name, dmg_taken);
        result += &format!("**{}** got healed by {} ❤️ \n", e.name, healed_amt);
      }
    }

    
    // Resolve triggered abilities
    for (ability, source, target) in ability_queue {
      let e: &mut Entity;
      match ability.effect.target {
        AbilityEffectTarget::This => {
          e = self.get_mut_entity_from_id(source).unwrap();
          result += &e.take_damage(ability.effect.damage, ability.effect.t).0;
        },
        AbilityEffectTarget::TriggerTarget => {
          e = self.get_mut_entity_from_id(target).unwrap();
          result += &e.take_damage(ability.effect.damage, ability.effect.t).0;
        },
        _ => {}
      }
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
}
