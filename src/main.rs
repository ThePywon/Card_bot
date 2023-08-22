use std::sync::{Arc, Mutex};

use dotenvy;
use poise::serenity_prelude as serenity;

mod scene;
mod entity;
mod events;
use entity::ability::{AbilityTriggerType, trigger_target::{ITSELF, ALLY, OPPONENT}, AbilityEffectTarget};
use scene::Scene;
use entity::{Attack, Ability, EntityBuilder};
use entity::dmg_type::{PHYSICAL, POISON, ACID, VAMPIRIC};
use entity::attributes;
use entity::{AbilityTrigger, AbilityEffect};
use entity::dmg_resistance::DamageResistance::{RESISTANCE, IMMUNITY};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, (Arc<Mutex<Scene>>, Arc<Mutex<Vec<EntityBuilder>>>), Error>;

#[tokio::main]
async fn main() { 
  dotenvy::dotenv().expect("Could not load environment variables.");
  let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
      commands: vec![
        describe_scene(),
        attack(),
        heal(),
        describe_entity(),
        nuke(),
        fill_scene()
      ],
      ..Default::default()
    })
    .token(std::env::var("TOKEN").expect("missing TOKEN"))
    .intents(serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::GUILD_MESSAGES)
    .setup(|ctx, _ready, framework| {
      Box::pin(async move {
        poise::builtins::register_globally(ctx, &framework.options().commands).await?;

        let mut species: Vec<EntityBuilder> = Vec::new();

        species.push(EntityBuilder::new(
          "Slime",
          "A small mass made out of a viscous substance.\n\
          It would almost be cute... if it wasn't able to eat you whole.",
          (2, 10), (attributes::NONE, vec![
            (PHYSICAL, RESISTANCE),
            (VAMPIRIC, RESISTANCE)
          ]),
          vec![
            (attributes::POISONOUS, vec![
              (POISON, IMMUNITY)
            ], 0.01),
            (attributes::ACIDIC, vec![
              (ACID, IMMUNITY)
            ], 0.01)],
          vec![
            &Ability { name: "Poison Touch", trigger: AbilityTrigger { t: AbilityTriggerType::Damage(PHYSICAL),
              source: ITSELF, target: OPPONENT },
              effect: AbilityEffect { target: AbilityEffectTarget::TriggerTarget, damage: 1, t: POISON },
              required_traits: attributes::POISONOUS, forbidden_traits: attributes::NONE, probability: 1.0 },
            &Ability { name: "Acid Touch", trigger: AbilityTrigger { t: AbilityTriggerType::Damage(PHYSICAL), source: ITSELF, target: OPPONENT},
              effect: AbilityEffect { target: AbilityEffectTarget::TriggerTarget, damage: 1, t: ACID },
              required_traits: attributes::ACIDIC, forbidden_traits: attributes::NONE, probability: 1.0 }
          ],
          vec![
            &Attack { name: "Head Bump", cost: 1, damage: 3, t: PHYSICAL, required_traits: attributes::NONE,
              forbidden_traits: attributes::NONE, prob: 1.0 }
          ]
        ));

        species.push(EntityBuilder::new(
          "Leech",
          "A small parasite that quite literally sucks the life force out of you!",
          (1, 5), (attributes::PHYSICAL, vec![
            (VAMPIRIC, IMMUNITY)
          ]),
          vec![(attributes::POISONOUS, Vec::new(), 0.1)],
          Vec::new(),
          vec![
            &Attack { name: "Parasite Bite", cost: 1, damage: 2, t: VAMPIRIC, required_traits: attributes::NONE,
            forbidden_traits: attributes::NONE, prob: 1.0 },
            &Attack { name: "Poison Sting", cost: 1, damage: 2, t: POISON, required_traits: attributes::POISONOUS,
            forbidden_traits: attributes::NONE, prob: 1.0 }
          ]
        ));

        species.push(EntityBuilder::new(
          "Bat",
          "One of the most iconic nocturnal creatures of Phunuse.",
          (1, 4), (attributes::PHYSICAL, vec![
            (ACID, RESISTANCE)
          ]),
          Vec::new(),
          vec![
            &Ability { name: "Echo Strike", trigger: AbilityTrigger { t: AbilityTriggerType::AnyDamage, source: ITSELF, target: ALLY | OPPONENT },
              effect: AbilityEffect { target: AbilityEffectTarget::TriggerTarget, damage: 1, t: PHYSICAL },
              required_traits: attributes::NONE, forbidden_traits: attributes::NONE, probability: 1.0 }
          ],
          vec![
            &Attack { name: "Sonic Scream", cost: 1, damage: 2, t: PHYSICAL, required_traits: attributes::NONE,
            forbidden_traits: attributes::NONE, prob: 1.0 }
          ]
        ));

        Ok((Arc::new(Mutex::from(Scene::new())), Arc::new(Mutex::from(species))))
      })
    });

  framework.run().await.unwrap();
}

/// Describe all entities currently in the scene
#[poise::command(slash_command,
  default_member_permissions = "SEND_MESSAGES",
  required_bot_permissions = "SEND_MESSAGES")]
async fn describe_scene(ctx: Context<'_>) -> Result<(), Error> {
  let mut result = Arc::clone(&ctx.data().0).lock().unwrap().describe_scene();
  if result.len() == 0 {
    result = String::from("Nothing in the scene yet!");
  }
  ctx.say(result).await?;
  Ok(())
}

/// Make an entity attack
#[poise::command(slash_command,
  default_member_permissions = "SEND_MESSAGES",
  required_bot_permissions = "SEND_MESSAGES",
  guild_only)]
async fn attack(ctx: Context<'_>,
  #[description = "Attack name"] attack_name: String,
  #[description = "Attacker ID"] attacker: u8,
  #[description = "Attack target ID"] target: u8) -> Result<(), Error> {
    let result = Arc::clone(&ctx.data().0).lock().unwrap().attack(&attack_name, attacker, target);
    ctx.say(result).await?;
    Ok(())
}

/// Heal an entity
#[poise::command(slash_command,
  default_member_permissions = "SEND_MESSAGES",
  required_bot_permissions = "SEND_MESSAGES",
  guild_only)]
async fn heal(ctx: Context<'_>,
  #[description = "Target entity ID"] target: u8,
  #[description = "Heal amount"] amount: u8) -> Result<(), Error> {
    let mut result = format!("Could not find entity with id #{}", target);
    if let Some(e) = Arc::clone(&ctx.data().0).lock().unwrap().get_mut_entity_from_id(target) {
      if e.is_alive() {
        let healed_amt = e.heal(amount);
        result = format!("Healed **{}**#{} for {} ❤️ ", e.name, target, healed_amt);
      }
      else {
        result = format!("Cannot heal **{}**#{} because it has already fainted!", e.name, target);
      }
    }
    ctx.say(result).await?;
    Ok(())
}

/// Describe an entity
#[poise::command(slash_command,
  default_member_permissions = "SEND_MESSAGES",
  required_bot_permissions = "SEND_MESSAGES")]
async fn describe_entity(ctx: Context<'_>,
  #[description = "Entity ID"] id: u8) -> Result<(), Error> {
    let mut result = format!("Could not find entity with id #{}", id);
    if let Some(e) = Arc::clone(&ctx.data().0).lock().unwrap().get_mut_entity_from_id(id) {
      result = e.describe();
    }
    ctx.say(result).await?;
    Ok(())
}

/// Literally nuke the scene
#[poise::command(slash_command,
  default_member_permissions = "SEND_MESSAGES",
  required_bot_permissions = "SEND_MESSAGES")]
async fn nuke(ctx: Context<'_>) -> Result<(), Error> {
  Arc::clone(&ctx.data().0).lock().unwrap().nuke();
  ctx.say("Nuke activated.\nCongrats, everything is gone now.").await?;
  Ok(())
}

/// Fill up the scene
#[poise::command(slash_command,
  default_member_permissions = "SEND_MESSAGES",
  required_bot_permissions = "SEND_MESSAGES")]
async fn fill_scene(ctx: Context<'_>) -> Result<(), Error> {
  if Arc::clone(&ctx.data().0).lock().unwrap().is_empty() {
    let slime = Arc::clone(&ctx.data().1).lock().unwrap()[0].build();
    let leech = Arc::clone(&ctx.data().1).lock().unwrap()[1].build();
    let bat = Arc::clone(&ctx.data().1).lock().unwrap()[2].build();

    Arc::clone(&ctx.data().0).lock().unwrap().register("A", &slime);
    Arc::clone(&ctx.data().0).lock().unwrap().register("B", &leech);
    Arc::clone(&ctx.data().0).lock().unwrap().register("A", &bat);

    ctx.say("Scene filled up!").await?;
  }
  else {
    ctx.say("Cannot fill scene because it is not empty!").await?;
  }
  Ok(())
}

