//! Game Simulation Integration Tests
//!
//! Tests focused on realistic game scenarios, demonstrating
//! ECS usage patterns in actual game development contexts.

use bemudjo_ecs::{Component, Query, SequentialSystemScheduler, System, World};

// Game Components
#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}
impl Component for Position {}

#[derive(Clone, Debug, PartialEq)]
struct Velocity {
    x: f32,
    y: f32,
}
impl Component for Velocity {}

#[derive(Clone, Debug, PartialEq)]
struct Health {
    current: i32,
    max: i32,
}
impl Component for Health {}

#[derive(Clone, Debug, PartialEq)]
struct Player {
    name: String,
    class: String,
}
impl Component for Player {}

#[derive(Clone, Debug, PartialEq)]
struct Enemy {
    enemy_type: String,
    damage: i32,
    attack_range: f32,
}
impl Component for Enemy {}

#[derive(Clone, Debug, PartialEq)]
struct Npc {
    name: String,
    dialogue: String,
    shop_items: Vec<String>,
}
impl Component for Npc {}

#[derive(Clone, Debug, PartialEq)]
struct Weapon {
    name: String,
    damage: i32,
    range: f32,
    durability: i32,
}
impl Component for Weapon {}

#[derive(Clone, Debug, PartialEq)]
struct Armor {
    name: String,
    defense: i32,
    durability: i32,
}
impl Component for Armor {}

#[derive(Clone, Debug, PartialEq)]
struct Experience {
    current: u64,
    level: u32,
}
impl Component for Experience {}

#[derive(Clone, Debug, PartialEq)]
struct Loot {
    items: Vec<String>,
    gold: u32,
}
impl Component for Loot {}

#[derive(Clone, Debug, PartialEq)]
struct Dead;
impl Component for Dead {}

#[derive(Clone, Debug, PartialEq)]
struct Projectile {
    damage: i32,
    speed: f32,
    lifetime: f32,
}
impl Component for Projectile {}

// Game Resources
#[derive(Debug, Clone, PartialEq)]
struct GameTime {
    elapsed: f64,
    delta: f32,
}
impl Component for GameTime {}

#[derive(Debug, Clone, PartialEq)]
struct GameStats {
    enemies_killed: u32,
    player_deaths: u32,
    items_collected: u32,
    total_damage_dealt: i32,
    session_time: f64,
}
impl Component for GameStats {}

#[derive(Debug, Clone, PartialEq)]
struct SpawnConfig {
    enemy_spawn_rate: f32,
    max_enemies: u32,
    spawn_locations: Vec<(f32, f32)>,
}
impl Component for SpawnConfig {}

// Game Systems

struct MovementSystem;

impl System for MovementSystem {
    fn run(&self, world: &mut World) {
        let delta = world
            .get_resource::<GameTime>()
            .map(|time| time.delta)
            .unwrap_or(0.016);

        let entities: Vec<_> = world.entities().cloned().collect();

        for entity in entities {
            if let (Some(pos), Some(vel)) = (
                world.get_component::<Position>(entity),
                world.get_component::<Velocity>(entity),
            ) {
                let new_pos = Position {
                    x: pos.x + vel.x * delta,
                    y: pos.y + vel.y * delta,
                };
                world.replace_component(entity, new_pos);
            }
        }
    }
}

struct CombatSystem;

impl System for CombatSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();
        let mut combat_events = Vec::new();

        // Find combat pairs
        for attacker in &entities {
            if world.has_component::<Dead>(*attacker) {
                continue;
            }

            let attacker_pos = match world.get_component::<Position>(*attacker) {
                Some(pos) => pos,
                None => continue,
            };

            let (attack_damage, attack_range) =
                if let Some(enemy) = world.get_component::<Enemy>(*attacker) {
                    (enemy.damage, enemy.attack_range)
                } else if world.has_component::<Player>(*attacker) {
                    let weapon_damage = world
                        .get_component::<Weapon>(*attacker)
                        .map(|w| w.damage)
                        .unwrap_or(10);
                    let weapon_range = world
                        .get_component::<Weapon>(*attacker)
                        .map(|w| w.range)
                        .unwrap_or(1.0);
                    (weapon_damage, weapon_range)
                } else {
                    continue;
                };

            // Find targets in range
            for target in &entities {
                if *target == *attacker || world.has_component::<Dead>(*target) {
                    continue;
                }

                // Players attack enemies, enemies attack players
                let valid_target = (world.has_component::<Player>(*attacker)
                    && world.has_component::<Enemy>(*target))
                    || (world.has_component::<Enemy>(*attacker)
                        && world.has_component::<Player>(*target));

                if !valid_target {
                    continue;
                }

                let target_pos = match world.get_component::<Position>(*target) {
                    Some(pos) => pos,
                    None => continue,
                };

                let distance = ((attacker_pos.x - target_pos.x).powi(2)
                    + (attacker_pos.y - target_pos.y).powi(2))
                .sqrt();

                if distance <= attack_range {
                    combat_events.push((*attacker, *target, attack_damage));
                }
            }
        }

        // Apply combat damage
        let mut total_damage = 0;
        for (_attacker, target, damage) in combat_events {
            if let Some(health) = world.get_component::<Health>(target) {
                let armor_reduction = world
                    .get_component::<Armor>(target)
                    .map(|a| a.defense)
                    .unwrap_or(0);

                let final_damage = (damage - armor_reduction).max(1);
                total_damage += final_damage;

                let new_health = Health {
                    current: health.current - final_damage,
                    max: health.max,
                };

                if new_health.current <= 0 {
                    world.add_component(target, Dead).ok();

                    // Grant experience to players
                    if world.has_component::<Player>(_attacker) {
                        let exp_gain = if world.has_component::<Enemy>(target) {
                            50
                        } else {
                            0
                        };

                        if exp_gain > 0 {
                            world
                                .update_resource::<GameStats, _>(|mut stats| {
                                    stats.enemies_killed += 1;
                                    stats
                                })
                                .ok();

                            world
                                .update_component::<Experience, _>(_attacker, |mut exp| {
                                    exp.current += exp_gain;
                                    if exp.current >= (exp.level as u64 + 1) * 100 {
                                        exp.level += 1;
                                        exp.current = 0;
                                    }
                                    exp
                                })
                                .ok();
                        }
                    }

                    // Create loot for dead enemies
                    if world.has_component::<Enemy>(target) {
                        let loot_items = vec!["Health Potion".to_string(), "Coin".to_string()];
                        world
                            .add_component(
                                target,
                                Loot {
                                    items: loot_items,
                                    gold: 10,
                                },
                            )
                            .ok();
                    }
                } else {
                    world.replace_component(target, new_health);
                }
            }
        }

        // Update total damage dealt
        if total_damage > 0 {
            world
                .update_resource::<GameStats, _>(|mut stats| {
                    stats.total_damage_dealt += total_damage;
                    stats
                })
                .ok();
        }
    }
}

struct EnemySpawnSystem;

impl System for EnemySpawnSystem {
    fn run(&self, world: &mut World) {
        // Clone resources before mutably borrowing world
        let time = world.get_resource::<GameTime>().cloned();
        let spawn_config = world.get_resource::<SpawnConfig>().cloned();

        if let (Some(time), Some(config)) = (time, spawn_config) {
            // Check if we should spawn
            if time.elapsed as f32 % config.enemy_spawn_rate < time.delta {
                let current_enemies = Query::<Enemy>::new()
                    .iter(world)
                    .filter(|(entity, _)| !world.has_component::<Dead>(*entity))
                    .count();

                if current_enemies < config.max_enemies as usize {
                    // Spawn new enemy
                    let spawn_index = (time.elapsed as usize) % config.spawn_locations.len();
                    let spawn_pos = config.spawn_locations[spawn_index];

                    let enemy = world.spawn_entity();
                    world
                        .add_component(
                            enemy,
                            Position {
                                x: spawn_pos.0,
                                y: spawn_pos.1,
                            },
                        )
                        .unwrap();
                    world
                        .add_component(
                            enemy,
                            Health {
                                current: 30,
                                max: 30,
                            },
                        )
                        .unwrap();
                    world
                        .add_component(
                            enemy,
                            Enemy {
                                enemy_type: "Goblin".to_string(),
                                damage: 15,
                                attack_range: 1.5,
                            },
                        )
                        .unwrap();
                    // Random movement
                    world
                        .add_component(
                            enemy,
                            Velocity {
                                x: (time.elapsed.sin() as f32) * 10.0,
                                y: (time.elapsed.cos() as f32) * 10.0,
                            },
                        )
                        .unwrap();
                }
            }
        }
    }
}

struct LootSystem;

impl System for LootSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();
        let mut collected_items = 0;

        for entity in entities {
            if !world.has_component::<Loot>(entity) {
                continue;
            }

            let loot_pos = match world.get_component::<Position>(entity) {
                Some(pos) => pos,
                None => continue,
            };

            // Check if any player is close enough to collect
            let player_entities: Vec<_> =
                Query::<Player>::new().iter(world).map(|(e, _)| e).collect();

            for player_entity in player_entities {
                if let Some(player_pos) = world.get_component::<Position>(player_entity) {
                    let distance = ((loot_pos.x - player_pos.x).powi(2)
                        + (loot_pos.y - player_pos.y).powi(2))
                    .sqrt();

                    if distance <= 2.0 {
                        // Collect loot
                        if let Some(loot) = world.get_component::<Loot>(entity) {
                            collected_items += loot.items.len() as u32;

                            // Grant experience for collecting items
                            world
                                .update_component::<Experience, _>(player_entity, |mut exp| {
                                    exp.current += 10; // Small exp for collecting
                                    exp
                                })
                                .ok();
                        }

                        world.remove_component::<Loot>(entity);
                        break;
                    }
                }
            }
        }

        if collected_items > 0 {
            world
                .update_resource::<GameStats, _>(|mut stats| {
                    stats.items_collected += collected_items;
                    stats
                })
                .ok();
        }
    }
}

struct CleanupSystem;

impl System for CleanupSystem {
    fn run(&self, world: &mut World) {
        let entities: Vec<_> = world.entities().cloned().collect();

        for entity in entities {
            if world.has_component::<Dead>(entity) {
                // Remove dead entities that don't have loot
                if !world.has_component::<Loot>(entity) {
                    world.delete_entity(entity);
                }
            }

            // Update projectile lifetime
            if world.has_component::<Projectile>(entity) {
                let delta = world
                    .get_resource::<GameTime>()
                    .map(|time| time.delta)
                    .unwrap_or(0.016);

                world
                    .update_component::<Projectile, _>(entity, |mut proj| {
                        proj.lifetime -= delta;
                        proj
                    })
                    .ok();

                if let Some(proj) = world.get_component::<Projectile>(entity) {
                    if proj.lifetime <= 0.0 {
                        world.delete_entity(entity);
                    }
                }
            }
        }
    }
}

struct TimeSystem;

impl System for TimeSystem {
    fn run(&self, world: &mut World) {
        if !world.has_resource::<GameTime>() {
            world.insert_resource(GameTime {
                elapsed: 0.0,
                delta: 0.016,
            });
        }

        world
            .update_resource::<GameTime, _>(|mut time| {
                time.elapsed += time.delta as f64;
                time
            })
            .unwrap();

        // Update session time in stats
        let elapsed = world
            .get_resource::<GameTime>()
            .map(|t| t.elapsed)
            .unwrap_or(0.0);
        world
            .update_resource::<GameStats, _>(|mut stats| {
                stats.session_time = elapsed;
                stats
            })
            .ok();
    }
}

#[test]
fn test_complete_rpg_combat_scenario() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Setup systems
    scheduler.add_system(TimeSystem).unwrap();
    scheduler.add_system(MovementSystem).unwrap();
    scheduler.add_system(EnemySpawnSystem).unwrap();
    scheduler.add_system(CombatSystem).unwrap();
    scheduler.add_system(LootSystem).unwrap();
    scheduler.add_system(CleanupSystem).unwrap();
    scheduler.build().unwrap();

    // Initialize resources
    world.insert_resource(GameStats {
        enemies_killed: 0,
        player_deaths: 0,
        items_collected: 0,
        total_damage_dealt: 0,
        session_time: 0.0,
    });

    world.insert_resource(SpawnConfig {
        enemy_spawn_rate: 2.0, // Every 2 seconds
        max_enemies: 5,
        spawn_locations: vec![(50.0, 50.0), (-50.0, 50.0), (50.0, -50.0), (-50.0, -50.0)],
    });

    // Create player
    let player = world.spawn_entity();
    world
        .add_component(player, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(player, Velocity { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(
            player,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();
    world
        .add_component(
            player,
            Player {
                name: "Hero".to_string(),
                class: "Warrior".to_string(),
            },
        )
        .unwrap();
    world
        .add_component(
            player,
            Experience {
                current: 0,
                level: 1,
            },
        )
        .unwrap();
    world
        .add_component(
            player,
            Weapon {
                name: "Iron Sword".to_string(),
                damage: 25,
                range: 2.0,
                durability: 100,
            },
        )
        .unwrap();
    world
        .add_component(
            player,
            Armor {
                name: "Leather Armor".to_string(),
                defense: 5,
                durability: 50,
            },
        )
        .unwrap();

    // Create some initial enemies
    for i in 0..3 {
        let enemy = world.spawn_entity();
        world
            .add_component(
                enemy,
                Position {
                    x: (i as f32 - 1.0) * 10.0,
                    y: 5.0,
                },
            )
            .unwrap();
        world
            .add_component(
                enemy,
                Health {
                    current: 30,
                    max: 30,
                },
            )
            .unwrap();
        world
            .add_component(
                enemy,
                Enemy {
                    enemy_type: "Orc".to_string(),
                    damage: 20,
                    attack_range: 1.5,
                },
            )
            .unwrap();
        world
            .add_component(
                enemy,
                Velocity {
                    x: 0.0,
                    y: -5.0, // Move toward player
                },
            )
            .unwrap();
    }

    // Run simulation
    for tick in 0..500 {
        // ~8 seconds of simulation
        scheduler.run_tick(&mut world);

        // Move player toward enemies periodically
        if tick % 60 == 0 {
            world
                .update_component::<Velocity, _>(player, |mut vel| {
                    vel.y = 2.0; // Move toward enemies
                    vel
                })
                .ok();
        }

        // Stop player movement if health is low
        if let Some(health) = world.get_component::<Health>(player) {
            if health.current < 30 {
                world
                    .update_component::<Velocity, _>(player, |mut vel| {
                        vel.x = 0.0;
                        vel.y = -10.0; // Retreat
                        vel
                    })
                    .ok();
            }
        }

        // Check if player died
        if world.has_component::<Dead>(player) {
            world
                .update_resource::<GameStats, _>(|mut stats| {
                    stats.player_deaths += 1;
                    stats
                })
                .ok();
            break;
        }

        // Cleanup deleted entities periodically
        if tick % 100 == 0 {
            world.cleanup_deleted_entities();
        }
    }

    // Verify game state
    let stats = world.get_resource::<GameStats>().unwrap();
    let time = world.get_resource::<GameTime>().unwrap();

    assert!(time.elapsed > 0.0);
    assert!(stats.session_time > 0.0);

    // Player should have interacted with the world
    assert!(stats.enemies_killed > 0 || stats.total_damage_dealt > 0 || stats.items_collected > 0);

    // Check player progression
    if !world.has_component::<Dead>(player) {
        let exp = world.get_component::<Experience>(player).unwrap();
        assert!(exp.current > 0 || exp.level > 1);
    }

    println!("Game Stats: {stats:?}");
    println!("Final time: {:.2}s", time.elapsed);
}

#[test]
fn test_mmo_like_scenario() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Setup systems for MMO-like scenario
    scheduler.add_system(TimeSystem).unwrap();
    scheduler.add_system(MovementSystem).unwrap();
    scheduler.add_system(CombatSystem).unwrap();
    scheduler.add_system(LootSystem).unwrap();
    scheduler.add_system(CleanupSystem).unwrap();
    scheduler.build().unwrap();

    // Initialize resources
    world.insert_resource(GameStats {
        enemies_killed: 0,
        player_deaths: 0,
        items_collected: 0,
        total_damage_dealt: 0,
        session_time: 0.0,
    });

    // Create multiple players (simulating MMO)
    let mut players = Vec::new();
    let player_classes = ["Warrior", "Mage", "Archer", "Healer"];

    for i in 0..10 {
        let player = world.spawn_entity();
        world
            .add_component(
                player,
                Position {
                    x: (i as f32 - 5.0) * 5.0,
                    y: 0.0,
                },
            )
            .unwrap();
        world
            .add_component(player, Velocity { x: 0.0, y: 0.0 })
            .unwrap();
        world
            .add_component(
                player,
                Health {
                    current: 100,
                    max: 100,
                },
            )
            .unwrap();
        world
            .add_component(
                player,
                Player {
                    name: format!("Player{i}"),
                    class: player_classes[i % player_classes.len()].to_string(),
                },
            )
            .unwrap();
        world
            .add_component(
                player,
                Experience {
                    current: 0,
                    level: 1,
                },
            )
            .unwrap();

        // Different equipment based on class
        match i % 4 {
            0 => {
                // Warrior
                world
                    .add_component(
                        player,
                        Weapon {
                            name: "Great Sword".to_string(),
                            damage: 30,
                            range: 2.0,
                            durability: 100,
                        },
                    )
                    .unwrap();
                world
                    .add_component(
                        player,
                        Armor {
                            name: "Plate Armor".to_string(),
                            defense: 10,
                            durability: 100,
                        },
                    )
                    .unwrap();
            }
            1 => {
                // Mage
                world
                    .add_component(
                        player,
                        Weapon {
                            name: "Magic Staff".to_string(),
                            damage: 40,
                            range: 5.0,
                            durability: 80,
                        },
                    )
                    .unwrap();
            }
            2 => {
                // Archer
                world
                    .add_component(
                        player,
                        Weapon {
                            name: "Long Bow".to_string(),
                            damage: 25,
                            range: 8.0,
                            durability: 60,
                        },
                    )
                    .unwrap();
            }
            3 => {
                // Healer
                world
                    .add_component(
                        player,
                        Weapon {
                            name: "Healing Staff".to_string(),
                            damage: 15,
                            range: 3.0,
                            durability: 90,
                        },
                    )
                    .unwrap();
            }
            _ => unreachable!(),
        }

        players.push(player);
    }

    // Create Npcs (towns, shops, etc.)
    for i in 0..5 {
        let npc = world.spawn_entity();
        world
            .add_component(
                npc,
                Position {
                    x: (i as f32 - 2.0) * 20.0,
                    y: -20.0,
                },
            )
            .unwrap();
        world
            .add_component(
                npc,
                Npc {
                    name: format!("Merchant{i}"),
                    dialogue: "Welcome to my shop!".to_string(),
                    shop_items: vec![
                        "Health Potion".to_string(),
                        "Mana Potion".to_string(),
                        "Iron Sword".to_string(),
                    ],
                },
            )
            .unwrap();
    }

    // Create boss enemy
    let boss = world.spawn_entity();
    world
        .add_component(boss, Position { x: 0.0, y: 30.0 })
        .unwrap();
    world
        .add_component(
            boss,
            Health {
                current: 500,
                max: 500,
            },
        )
        .unwrap();
    world
        .add_component(
            boss,
            Enemy {
                enemy_type: "Dragon".to_string(),
                damage: 50,
                attack_range: 5.0,
            },
        )
        .unwrap();
    world
        .add_component(boss, Velocity { x: 0.0, y: -1.0 })
        .unwrap();

    // Create multiple smaller enemies
    for i in 0..20 {
        let enemy = world.spawn_entity();
        world
            .add_component(
                enemy,
                Position {
                    x: ((i as f32 % 10.0) - 5.0) * 3.0,
                    y: 15.0 + (i as f32 / 10.0) * 5.0,
                },
            )
            .unwrap();
        world
            .add_component(
                enemy,
                Health {
                    current: 40,
                    max: 40,
                },
            )
            .unwrap();
        world
            .add_component(
                enemy,
                Enemy {
                    enemy_type: "Skeleton".to_string(),
                    damage: 15,
                    attack_range: 1.5,
                },
            )
            .unwrap();
        world
            .add_component(
                enemy,
                Velocity {
                    x: ((i as f32).sin()) * 2.0,
                    y: -3.0,
                },
            )
            .unwrap();
    }

    // Run raid simulation
    for tick in 0..1000 {
        // Longer simulation for MMO scenario
        scheduler.run_tick(&mut world);

        // Players coordinate movement toward enemies
        if tick % 30 == 0 {
            for (i, &player) in players.iter().enumerate() {
                if world.has_component::<Dead>(player) {
                    continue;
                }

                // Different movement strategies by class
                let movement = match i % 4 {
                    0 => (0.0, 5.0),  // Warriors charge forward
                    1 => (2.0, 3.0),  // Mages stay at range
                    2 => (-2.0, 3.0), // Archers kite
                    3 => (0.0, 1.0),  // Healers stay back
                    _ => (0.0, 0.0),
                };

                world
                    .update_component::<Velocity, _>(player, |mut vel| {
                        vel.x = movement.0;
                        vel.y = movement.1;
                        vel
                    })
                    .ok();
            }
        }

        // Check boss health
        if let Some(boss_health) = world.get_component::<Health>(boss) {
            if boss_health.current <= 0 && !world.has_component::<Dead>(boss) {
                // Boss defeated - create epic loot
                world
                    .add_component(
                        boss,
                        Loot {
                            items: vec![
                                "Dragon Scale".to_string(),
                                "Epic Sword".to_string(),
                                "Dragon Heart".to_string(),
                            ],
                            gold: 1000,
                        },
                    )
                    .ok();

                // Grant experience to all living players
                for &player in &players {
                    if !world.has_component::<Dead>(player) {
                        world
                            .update_component::<Experience, _>(player, |mut exp| {
                                exp.current += 500; // Boss kill bonus
                                while exp.current >= (exp.level as u64 + 1) * 100 {
                                    exp.level += 1;
                                    exp.current -= exp.level as u64 * 100;
                                }
                                exp
                            })
                            .ok();
                    }
                }
                break;
            }
        }

        // Cleanup periodically
        if tick % 200 == 0 {
            world.cleanup_deleted_entities();
        }
    }

    // Verify MMO scenario results
    let stats = world.get_resource::<GameStats>().unwrap();

    // Should have significant activity
    assert!(stats.enemies_killed > 5);
    assert!(stats.total_damage_dealt > 500);

    // Check player progression
    let mut total_levels = 0;
    let mut living_players = 0;

    for &player in &players {
        if !world.has_component::<Dead>(player) {
            living_players += 1;
            if let Some(exp) = world.get_component::<Experience>(player) {
                total_levels += exp.level;
            }
        }
    }

    assert!(living_players > 0); // Some players should survive
    assert!(total_levels >= 10); // Players should have leveled up significantly

    // Npcs should still exist
    let npc_count = Query::<Npc>::new().iter(&world).count();
    assert_eq!(npc_count, 5);
}

#[test]
fn test_survival_game_scenario() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Resource for hunger/thirst mechanics
    #[derive(Debug, Clone, PartialEq)]
    struct Survival {
        hunger: f32,
        thirst: f32,
        temperature: f32,
    }
    impl Component for Survival {}

    // Survival system
    struct SurvivalSystem;
    impl System for SurvivalSystem {
        fn run(&self, world: &mut World) {
            let delta = world
                .get_resource::<GameTime>()
                .map(|time| time.delta)
                .unwrap_or(0.016);

            let entities: Vec<_> = world.entities().cloned().collect();

            for entity in entities {
                if world.has_component::<Survival>(entity) && world.has_component::<Player>(entity)
                {
                    world
                        .update_component::<Survival, _>(entity, |mut survival| {
                            survival.hunger += delta * 2.0; // Hunger increases over time
                            survival.thirst += delta * 3.0; // Thirst increases faster
                            survival
                        })
                        .ok();

                    // Apply survival effects
                    if let Some(survival) = world.get_component::<Survival>(entity) {
                        if survival.hunger > 80.0 || survival.thirst > 80.0 {
                            // Take damage from hunger/thirst
                            world
                                .update_component::<Health, _>(entity, |mut health| {
                                    health.current -= 1;
                                    health
                                })
                                .ok();
                        }
                    }
                }
            }
        }
    }

    // Setup systems
    scheduler.add_system(TimeSystem).unwrap();
    scheduler.add_system(MovementSystem).unwrap();
    scheduler.add_system(SurvivalSystem).unwrap();
    scheduler.add_system(CombatSystem).unwrap();
    scheduler.add_system(LootSystem).unwrap();
    scheduler.add_system(CleanupSystem).unwrap();
    scheduler.build().unwrap();

    // Initialize resources
    world.insert_resource(GameStats {
        enemies_killed: 0,
        player_deaths: 0,
        items_collected: 0,
        total_damage_dealt: 0,
        session_time: 0.0,
    });

    // Create player with survival needs
    let player = world.spawn_entity();
    world
        .add_component(player, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(player, Velocity { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(
            player,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();
    world
        .add_component(
            player,
            Player {
                name: "Survivor".to_string(),
                class: "Explorer".to_string(),
            },
        )
        .unwrap();
    world
        .add_component(
            player,
            Survival {
                hunger: 0.0,
                thirst: 0.0,
                temperature: 20.0,
            },
        )
        .unwrap();

    // Create resource nodes (food, water sources)
    let food_sources = vec![(10.0, 10.0), (-15.0, 5.0), (8.0, -12.0)];

    for (x, y) in food_sources {
        let food = world.spawn_entity();
        world.add_component(food, Position { x, y }).unwrap();
        world
            .add_component(
                food,
                Loot {
                    items: vec!["Berries".to_string(), "Water".to_string()],
                    gold: 0,
                },
            )
            .unwrap();
    }

    // Create hostile wildlife
    for i in 0..8 {
        let animal = world.spawn_entity();
        world
            .add_component(
                animal,
                Position {
                    x: (i as f32 - 4.0) * 8.0,
                    y: (i as f32 % 3.0 - 1.0) * 12.0,
                },
            )
            .unwrap();
        world
            .add_component(
                animal,
                Health {
                    current: 25,
                    max: 25,
                },
            )
            .unwrap();
        world
            .add_component(
                animal,
                Enemy {
                    enemy_type: "Wolf".to_string(),
                    damage: 12,
                    attack_range: 2.0,
                },
            )
            .unwrap();
        world
            .add_component(
                animal,
                Velocity {
                    x: ((i as f32).sin()) * 3.0,
                    y: ((i as f32).cos()) * 3.0,
                },
            )
            .unwrap();
    }

    // Run survival simulation
    for tick in 0..2000 {
        // Longer simulation for survival
        scheduler.run_tick(&mut world);

        // Player movement AI - seek food when hungry
        if tick % 60 == 0 {
            // Check every second
            if let Some(survival) = world.get_component::<Survival>(player) {
                if survival.hunger > 50.0 || survival.thirst > 50.0 {
                    // Move toward nearest food source
                    world
                        .update_component::<Velocity, _>(player, |mut vel| {
                            vel.x = 2.0;
                            vel.y = 2.0;
                            vel
                        })
                        .ok();
                } else {
                    // Explore randomly
                    world
                        .update_component::<Velocity, _>(player, |mut vel| {
                            vel.x = ((tick as f32 / 100.0).sin()) * 5.0;
                            vel.y = ((tick as f32 / 100.0).cos()) * 5.0;
                            vel
                        })
                        .ok();
                }
            }
        }

        // Reduce survival needs when collecting food
        if tick % 100 == 0 {
            if let Some(survival) = world.get_component::<Survival>(player) {
                if survival.hunger > 60.0 || survival.thirst > 60.0 {
                    // Simulate consuming resources
                    world
                        .update_component::<Survival, _>(player, |mut survival| {
                            survival.hunger = (survival.hunger - 30.0).max(0.0);
                            survival.thirst = (survival.thirst - 40.0).max(0.0);
                            survival
                        })
                        .ok();
                }
            }
        }

        // Check death conditions
        if world.has_component::<Dead>(player) {
            world
                .update_resource::<GameStats, _>(|mut stats| {
                    stats.player_deaths += 1;
                    stats
                })
                .ok();
            break;
        }

        // Cleanup
        if tick % 300 == 0 {
            world.cleanup_deleted_entities();
        }
    }

    // Verify survival scenario
    let stats = world.get_resource::<GameStats>().unwrap();
    let time = world.get_resource::<GameTime>().unwrap();

    assert!(time.elapsed > 10.0); // Should run for a while

    // Check if player survived or died from survival needs
    if world.has_component::<Dead>(player) {
        assert_eq!(stats.player_deaths, 1);
    } else {
        // Player survived - check final survival state
        let survival = world.get_component::<Survival>(player).unwrap();
        assert!(survival.hunger < 100.0);
        assert!(survival.thirst < 100.0);
    }

    // Should have collected some items
    assert!(stats.items_collected > 0);

    println!("Survival Stats: {stats:?}");
    println!("Survival time: {:.2}s", time.elapsed);

    if !world.has_component::<Dead>(player) {
        let survival = world.get_component::<Survival>(player).unwrap();
        println!("Final survival state: {survival:?}");
    }
}

#[test]
fn test_tower_defense_scenario() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();

    // Tower defense specific components
    #[derive(Clone, Debug, PartialEq)]
    struct Tower {
        damage: i32,
        range: f32,
        fire_rate: f32,
        last_shot: f64,
    }
    impl Component for Tower {}

    #[derive(Clone, Debug, PartialEq)]
    struct Waypoint {
        next_waypoint: Option<(f32, f32)>,
    }
    impl Component for Waypoint {}

    // Tower shooting system
    struct TowerSystem;
    impl System for TowerSystem {
        fn run(&self, world: &mut World) {
            let time = world
                .get_resource::<GameTime>()
                .map(|t| t.elapsed)
                .unwrap_or(0.0);

            let tower_entities: Vec<_> =
                Query::<Tower>::new().iter(world).map(|(e, _)| e).collect();

            let enemy_entities: Vec<_> = Query::<Enemy>::new()
                .iter(world)
                .filter(|(e, _)| !world.has_component::<Dead>(*e))
                .map(|(e, _)| e)
                .collect();

            for tower_entity in tower_entities {
                let tower_pos = match world.get_component::<Position>(tower_entity) {
                    Some(pos) => *pos, // Copy Position (derive Copy if not already)
                    None => continue,
                };
                let mut tower = match world.get_component::<Tower>(tower_entity) {
                    Some(tower) => tower.clone(),
                    None => continue,
                };
                // Check if tower can shoot
                if time - tower.last_shot < tower.fire_rate as f64 {
                    continue;
                }
                // Find closest enemy in range
                let mut closest_enemy = None;
                let mut closest_distance = tower.range;
                for enemy_entity in &enemy_entities {
                    if let Some(enemy_pos) = world.get_component::<Position>(*enemy_entity) {
                        let distance = ((tower_pos.x - enemy_pos.x).powi(2)
                            + (tower_pos.y - enemy_pos.y).powi(2))
                        .sqrt();
                        if distance <= closest_distance {
                            closest_distance = distance;
                            closest_enemy = Some(*enemy_entity);
                        }
                    }
                }
                // Shoot at closest enemy
                if let Some(target) = closest_enemy {
                    tower.last_shot = time;
                    world.replace_component(tower_entity, tower.clone());
                    // Create projectile
                    let projectile = world.spawn_entity();
                    world
                        .add_component(
                            projectile,
                            Position {
                                x: tower_pos.x,
                                y: tower_pos.y,
                            },
                        )
                        .unwrap();

                    // Calculate projectile velocity toward target
                    if let Some(target_pos) = world.get_component::<Position>(target) {
                        let dx = target_pos.x - tower_pos.x;
                        let dy = target_pos.y - tower_pos.y;
                        let length = (dx * dx + dy * dy).sqrt();

                        world
                            .add_component(
                                projectile,
                                Velocity {
                                    x: (dx / length) * 50.0,
                                    y: (dy / length) * 50.0,
                                },
                            )
                            .unwrap();
                    }

                    world
                        .add_component(
                            projectile,
                            Projectile {
                                damage: tower.damage,
                                speed: 50.0,
                                lifetime: 2.0,
                            },
                        )
                        .unwrap();
                }
            }
        }
    }

    // Projectile collision system
    struct ProjectileSystem;
    impl System for ProjectileSystem {
        fn run(&self, world: &mut World) {
            let projectile_entities: Vec<_> = Query::<Projectile>::new()
                .iter(world)
                .map(|(e, _)| e)
                .collect();

            let enemy_entities: Vec<_> = Query::<Enemy>::new()
                .iter(world)
                .filter(|(e, _)| !world.has_component::<Dead>(*e))
                .map(|(e, _)| e)
                .collect();

            for projectile_entity in projectile_entities {
                let proj_pos = match world.get_component::<Position>(projectile_entity) {
                    Some(pos) => pos,
                    None => continue,
                };

                let projectile = match world.get_component::<Projectile>(projectile_entity) {
                    Some(proj) => proj.clone(),
                    None => continue,
                };

                // Check collision with enemies
                for enemy_entity in &enemy_entities {
                    if let Some(enemy_pos) = world.get_component::<Position>(*enemy_entity) {
                        let distance = ((proj_pos.x - enemy_pos.x).powi(2)
                            + (proj_pos.y - enemy_pos.y).powi(2))
                        .sqrt();

                        if distance <= 1.0 {
                            // Hit
                            // Damage enemy
                            world
                                .update_component::<Health, _>(*enemy_entity, |mut health| {
                                    health.current -= projectile.damage;
                                    health
                                })
                                .ok();

                            // Check if enemy died
                            if let Some(health) = world.get_component::<Health>(*enemy_entity) {
                                if health.current <= 0 {
                                    world.add_component(*enemy_entity, Dead).ok();

                                    world
                                        .update_resource::<GameStats, _>(|mut stats| {
                                            stats.enemies_killed += 1;
                                            stats
                                        })
                                        .ok();
                                }
                            }

                            // Remove projectile
                            world.delete_entity(projectile_entity);
                            break;
                        }
                    }
                }
            }
        }
    }

    // Enemy spawning and movement system
    struct WaveSystem {
        wave_number: u32,
        enemies_spawned: u32,
        enemies_per_wave: u32,
        last_spawn: f64,
        spawn_rate: f32,
    }

    impl WaveSystem {
        fn new() -> Self {
            Self {
                wave_number: 1,
                enemies_spawned: 0,
                enemies_per_wave: 10,
                last_spawn: 0.0,
                spawn_rate: 1.0,
            }
        }
    }

    impl System for WaveSystem {
        fn run(&self, world: &mut World) {
            let time = world
                .get_resource::<GameTime>()
                .map(|t| t.elapsed)
                .unwrap_or(0.0);

            // Check if we should spawn more enemies
            if self.enemies_spawned < self.enemies_per_wave
                && time - self.last_spawn >= self.spawn_rate as f64
            {
                let enemy = world.spawn_entity();
                world
                    .add_component(enemy, Position { x: -50.0, y: 0.0 })
                    .unwrap();
                world
                    .add_component(enemy, Velocity { x: 10.0, y: 0.0 })
                    .unwrap();
                world
                    .add_component(
                        enemy,
                        Health {
                            current: 20 + (self.wave_number * 5) as i32,
                            max: 20 + (self.wave_number * 5) as i32,
                        },
                    )
                    .unwrap();
                world
                    .add_component(
                        enemy,
                        Enemy {
                            enemy_type: format!("Wave{}_Enemy", self.wave_number),
                            damage: 10 + self.wave_number as i32,
                            attack_range: 1.0,
                        },
                    )
                    .unwrap();
                world
                    .add_component(
                        enemy,
                        Waypoint {
                            next_waypoint: Some((50.0, 0.0)), // Goal position
                        },
                    )
                    .unwrap();
            }
        }
    }

    // Setup systems
    scheduler.add_system(TimeSystem).unwrap();
    scheduler.add_system(MovementSystem).unwrap();
    scheduler.add_system(WaveSystem::new()).unwrap();
    scheduler.add_system(TowerSystem).unwrap();
    scheduler.add_system(ProjectileSystem).unwrap();
    scheduler.add_system(CleanupSystem).unwrap();
    scheduler.build().unwrap();

    // Initialize resources
    world.insert_resource(GameStats {
        enemies_killed: 0,
        player_deaths: 0,
        items_collected: 0,
        total_damage_dealt: 0,
        session_time: 0.0,
    });

    // Create towers
    let tower_positions = vec![
        (-20.0, 10.0),
        (0.0, 15.0),
        (20.0, 10.0),
        (-20.0, -10.0),
        (20.0, -10.0),
    ];

    for pos in tower_positions {
        let tower = world.spawn_entity();
        world
            .add_component(tower, Position { x: pos.0, y: pos.1 })
            .unwrap();
        world
            .add_component(
                tower,
                Tower {
                    damage: 25,
                    range: 15.0,
                    fire_rate: 0.5,
                    last_shot: 0.0,
                },
            )
            .unwrap();
    }

    // Run tower defense simulation
    for tick in 0..1500 {
        // ~25 seconds
        scheduler.run_tick(&mut world);

        // Check if enemies reached the goal
        let enemy_entities: Vec<_> = Query::<Enemy>::new()
            .iter(&world)
            .filter(|(e, _)| !world.has_component::<Dead>(*e))
            .map(|(e, _)| e)
            .collect();

        for enemy_entity in enemy_entities {
            if let Some(pos) = world.get_component::<Position>(enemy_entity) {
                if pos.x >= 45.0 {
                    // Reached goal
                    world.delete_entity(enemy_entity);
                    world
                        .update_resource::<GameStats, _>(|mut stats| {
                            stats.player_deaths += 1; // Count as life lost
                            stats
                        })
                        .ok();
                }
            }
        }

        // Cleanup
        if tick % 100 == 0 {
            world.cleanup_deleted_entities();
        }
    }

    // Verify tower defense results
    let stats = world.get_resource::<GameStats>().unwrap();

    // Should have killed many enemies
    assert!(stats.enemies_killed > 5);

    // Check tower count
    let tower_count = Query::<Tower>::new().iter(&world).count();
    assert_eq!(tower_count, 5);

    // Check projectile system worked
    let projectile_count = Query::<Projectile>::new().iter(&world).count();
    // May or may not have projectiles at end

    println!("Tower Defense Stats: {stats:?}");
    println!(
        "Towers: {tower_count}, Active projectiles: {projectile_count}"
    );
}
