use super::DamageType;
pub mod trigger_target;
use trigger_target::{ITSELF, ALLY, OPPONENT, CURRENT_TEAM, ALL_BUT_ALLY, ALL_BUT_SELF, ALL};
use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub struct Ability {
  pub name: &'static str,
  pub trigger: AbilityTrigger,
  pub effect: AbilityEffect,
  pub required_traits: u8,
  pub forbidden_traits: u8,
  pub probability: f32
}

impl Display for Ability {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "  \\# `{}`\n    {}, {}.\n", self.name, self.trigger, self.effect)
  }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub struct AbilityTrigger {
  pub t: AbilityTriggerType,
  pub source: u8,
  pub target: u8
}

impl AbilityTrigger {
  pub fn match_(&self, other: &AbilityTrigger) -> bool {
    self.t == other.t && self.source & other.source != 0 &&
    self.target & other.target != 0
  }
}

#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum AbilityTriggerType {
  AnyDamage,
  Damage(DamageType),
  Heal
}

#[derive(Debug, Clone, Copy)]
pub struct AbilityEffect {
  pub target: AbilityEffectTarget,
  pub damage: u8,
  pub t: DamageType
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum AbilityEffectTarget {
  Itself,
  TriggerTarget,
  AnyAlly,
  AnyOpponent,
  AllAlly,
  AllOpponent
}

impl Display for AbilityTrigger {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut result = String::new();
    let mut source_is_this = false;

    let source = self.source & ALL;
    let target = self.target & ALL;

    result += match source {
      ITSELF => {
        source_is_this = true;
        "Upon "
      },
      ALLY => "Whenever an ally ",
      OPPONENT => "Whenever an opponent ",
      CURRENT_TEAM => "Whenever any creature on this team ",
      ALL_BUT_ALLY => "Whenever itself or an opponent ",
      ALL_BUT_SELF => "Whenever an ally or an opponent ",
      ALL | _ => "Whenever any creature "
    };


    if source_is_this {
      match self.t {
        AbilityTriggerType::AnyDamage => result += "dealing damage to ",
        AbilityTriggerType::Damage(dt) => result += &format!("dealing {} damage to ", dt),
        AbilityTriggerType::Heal => result += "healing "
      }
    }
    else {
      match self.t {
        AbilityTriggerType::AnyDamage => result += "deals damage to ",
        AbilityTriggerType::Damage(dt) => result += &format!("deals {} damage to ", dt),
        AbilityTriggerType::Heal => result += "heals "
      }
    }


    result += match target {
      ITSELF => {
        if source_is_this { "itself" }
        else { "this creature" }
      },
      ALLY => "an ally",
      OPPONENT => "an opponent",
      CURRENT_TEAM => "any creature in this team",
      ALL_BUT_ALLY => "this creature or an ally",
      ALL_BUT_SELF => "an ally or an opponent",
      ALL | _ => "any creature"
    };


    write!(f, "{}", result)
  }
}

impl Display for AbilityEffect {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let target_str: &str;

    match self.target {
      AbilityEffectTarget::Itself => target_str = "itself",
      AbilityEffectTarget::TriggerTarget => target_str = "that targeted creature",
      AbilityEffectTarget::AnyAlly => target_str = "a random ally",
      AbilityEffectTarget::AnyOpponent => target_str = "a random opponent",
      AbilityEffectTarget::AllAlly => target_str = "all allies",
      AbilityEffectTarget::AllOpponent => target_str = "all opponents"
    }

    write!(f, "deal {} {} damage to {}", self.damage, self.t, target_str)
  }
}
