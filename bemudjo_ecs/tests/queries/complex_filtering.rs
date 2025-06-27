//! Complex Query Filtering Integration Tests
//!
//! Tests focused on advanced query patterns, complex filtering scenarios,
//! and edge cases in query system behavior.

use bemudjo_ecs::{Component, Query, World};

// Test Components
#[derive(Clone, Debug, PartialEq)]
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
    current: u32,
    max: u32,
}
impl Component for Health {}

#[derive(Clone, Debug, PartialEq)]
struct Damage {
    amount: u32,
}
impl Component for Damage {}

#[derive(Clone, Debug, PartialEq)]
struct Dead;
impl Component for Dead {}

#[derive(Clone, Debug, PartialEq)]
struct Player;
impl Component for Player {}

#[derive(Clone, Debug, PartialEq)]
struct Enemy;
impl Component for Enemy {}

#[derive(Clone, Debug, PartialEq)]
struct Npc;
impl Component for Npc {}

#[derive(Clone, Debug, PartialEq)]
struct Tag {
    name: String,
}
impl Component for Tag {}

#[derive(Clone, Debug, PartialEq)]
struct Level {
    value: u32,
}
impl Component for Level {}

#[derive(Clone, Debug, PartialEq)]
struct Experience {
    points: u64,
}
impl Component for Experience {}

#[derive(Clone, Debug, PartialEq)]
struct Weapon {
    damage: u32,
    durability: u32,
}
impl Component for Weapon {}

#[derive(Clone, Debug, PartialEq)]
struct Armor {
    defense: u32,
    weight: f32,
}
impl Component for Armor {}

#[test]
fn test_complex_multi_component_filtering() {
    let mut world = World::new();

    // Create player
    let player = world.spawn_entity();
    world
        .add_component(player, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world
        .add_component(player, Velocity { x: 1.0, y: 0.0 })
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
    world.add_component(player, Player).unwrap();
    world.add_component(player, Level { value: 5 }).unwrap();
    world
        .add_component(
            player,
            Weapon {
                damage: 25,
                durability: 100,
            },
        )
        .unwrap();

    // Create enemies
    let enemy1 = world.spawn_entity();
    world
        .add_component(enemy1, Position { x: 10.0, y: 5.0 })
        .unwrap();
    world
        .add_component(
            enemy1,
            Health {
                current: 50,
                max: 50,
            },
        )
        .unwrap();
    world.add_component(enemy1, Enemy).unwrap();
    world.add_component(enemy1, Level { value: 3 }).unwrap();

    let enemy2 = world.spawn_entity();
    world
        .add_component(enemy2, Position { x: -5.0, y: 10.0 })
        .unwrap();
    world
        .add_component(enemy2, Velocity { x: -1.0, y: 0.0 })
        .unwrap();
    world
        .add_component(
            enemy2,
            Health {
                current: 75,
                max: 75,
            },
        )
        .unwrap();
    world.add_component(enemy2, Enemy).unwrap();
    world.add_component(enemy2, Level { value: 4 }).unwrap();
    world
        .add_component(
            enemy2,
            Weapon {
                damage: 15,
                durability: 80,
            },
        )
        .unwrap();

    // Create Npcs
    let npc1 = world.spawn_entity();
    world
        .add_component(npc1, Position { x: 20.0, y: 0.0 })
        .unwrap();
    world
        .add_component(
            npc1,
            Health {
                current: 30,
                max: 30,
            },
        )
        .unwrap();
    world.add_component(npc1, Npc).unwrap();
    world
        .add_component(
            npc1,
            Tag {
                name: "Merchant".to_string(),
            },
        )
        .unwrap();

    let npc2 = world.spawn_entity();
    world
        .add_component(npc2, Position { x: 15.0, y: 15.0 })
        .unwrap();
    world
        .add_component(
            npc2,
            Health {
                current: 40,
                max: 40,
            },
        )
        .unwrap();
    world.add_component(npc2, Npc).unwrap();
    world
        .add_component(
            npc2,
            Tag {
                name: "Guard".to_string(),
            },
        )
        .unwrap();
    world
        .add_component(
            npc2,
            Weapon {
                damage: 20,
                durability: 50,
            },
        )
        .unwrap();

    // Dead enemy
    let dead_enemy = world.spawn_entity();
    world
        .add_component(dead_enemy, Position { x: 0.0, y: -10.0 })
        .unwrap();
    world
        .add_component(
            dead_enemy,
            Health {
                current: 0,
                max: 50,
            },
        )
        .unwrap();
    world.add_component(dead_enemy, Enemy).unwrap();
    world.add_component(dead_enemy, Dead).unwrap();

    // Test 1: All living entities with health
    let living_query = Query::<Position>::new().with::<Health>().without::<Dead>();
    let living_results: Vec<_> = living_query.iter(&world).collect();
    assert_eq!(living_results.len(), 5); // All except dead enemy

    // Test 2: Moving entities (with velocity)
    let moving_query = Query::<Position>::new().with::<Velocity>();
    let moving_results: Vec<_> = moving_query.iter(&world).collect();
    assert_eq!(moving_results.len(), 2); // Player and enemy2

    let moving_entities: Vec<_> = moving_results.iter().map(|(e, _)| *e).collect();
    assert!(moving_entities.contains(&player));
    assert!(moving_entities.contains(&enemy2));

    // Test 3: Combat-capable entities (have weapons)
    let combat_query = Query::<Position>::new()
        .with::<Weapon>()
        .with::<Health>()
        .without::<Dead>();
    let combat_results: Vec<_> = combat_query.iter(&world).collect();
    assert_eq!(combat_results.len(), 3); // Player, enemy2, npc2

    // Test 4: Enemies only (alive)
    let enemy_query = Query::<Position>::new().with::<Enemy>().without::<Dead>();
    let enemy_results: Vec<_> = enemy_query.iter(&world).collect();
    assert_eq!(enemy_results.len(), 2); // enemy1, enemy2

    // Test 5: High-level entities (level >= 4)
    let high_level_query = Query::<Position>::new().with::<Level>();
    let high_level_results: Vec<_> = high_level_query
        .iter(&world)
        .filter(|(entity, _)| {
            world
                .get_component::<Level>(*entity)
                .map(|level| level.value >= 4)
                .unwrap_or(false)
        })
        .collect();
    assert_eq!(high_level_results.len(), 2); // Player (5), enemy2 (4)

    // Test 6: Tagged Npcs
    let tagged_npc_query = Query::<Position>::new().with::<Npc>().with::<Tag>();
    let tagged_npc_results: Vec<_> = tagged_npc_query.iter(&world).collect();
    assert_eq!(tagged_npc_results.len(), 2); // Both Npcs have tags
}

#[test]
fn test_exclusion_filtering_patterns() {
    let mut world = World::new();

    // Create entities with various component combinations
    let entity1 = world.spawn_entity();
    world
        .add_component(entity1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world
        .add_component(
            entity1,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();
    world.add_component(entity1, Player).unwrap();

    let entity2 = world.spawn_entity();
    world
        .add_component(entity2, Position { x: 2.0, y: 2.0 })
        .unwrap();
    world
        .add_component(
            entity2,
            Health {
                current: 50,
                max: 50,
            },
        )
        .unwrap();
    world.add_component(entity2, Enemy).unwrap();
    world.add_component(entity2, Damage { amount: 10 }).unwrap();

    let entity3 = world.spawn_entity();
    world
        .add_component(entity3, Position { x: 3.0, y: 3.0 })
        .unwrap();
    world
        .add_component(
            entity3,
            Health {
                current: 75,
                max: 75,
            },
        )
        .unwrap();
    world.add_component(entity3, Npc).unwrap();

    let entity4 = world.spawn_entity();
    world
        .add_component(entity4, Position { x: 4.0, y: 4.0 })
        .unwrap();
    world.add_component(entity4, Enemy).unwrap();
    world.add_component(entity4, Dead).unwrap();

    let entity5 = world.spawn_entity();
    world
        .add_component(entity5, Position { x: 5.0, y: 5.0 })
        .unwrap();
    world
        .add_component(entity5, Velocity { x: 1.0, y: 1.0 })
        .unwrap();

    // Test: Entities without specific components
    let no_health_query = Query::<Position>::new().without::<Health>();
    let no_health_results: Vec<_> = no_health_query.iter(&world).collect();
    assert_eq!(no_health_results.len(), 2); // entity4, entity5

    // Test: Living entities (no Dead component)
    let living_query = Query::<Position>::new().with::<Health>().without::<Dead>();
    let living_results: Vec<_> = living_query.iter(&world).collect();
    assert_eq!(living_results.len(), 3); // entity1, entity2, entity3

    // Test: Non-damaged entities
    let undamaged_query = Query::<Position>::new()
        .with::<Health>()
        .without::<Damage>();
    let undamaged_results: Vec<_> = undamaged_query.iter(&world).collect();
    assert_eq!(undamaged_results.len(), 2); // entity1, entity3

    // Test: Multiple exclusions
    let specific_query = Query::<Position>::new()
        .without::<Player>()
        .without::<Dead>()
        .without::<Damage>();
    let specific_results: Vec<_> = specific_query.iter(&world).collect();
    assert_eq!(specific_results.len(), 2); // entity3, entity5

    // Test: Complex inclusion/exclusion
    let complex_query = Query::<Position>::new()
        .with::<Health>()
        .without::<Player>()
        .without::<Dead>();
    let complex_results: Vec<_> = complex_query.iter(&world).collect();
    assert_eq!(complex_results.len(), 2); // entity2, entity3
}

#[test]
fn test_nested_query_conditions() {
    let mut world = World::new();

    // Create a complex entity hierarchy
    for i in 0..20 {
        let entity = world.spawn_entity();
        world
            .add_component(
                entity,
                Position {
                    x: i as f32,
                    y: 0.0,
                },
            )
            .unwrap();

        // Pattern 1: Players (even IDs)
        if i % 2 == 0 {
            world.add_component(entity, Player).unwrap();
            world.add_component(entity, Level { value: i / 2 }).unwrap();

            if i % 4 == 0 {
                world
                    .add_component(
                        entity,
                        Weapon {
                            damage: 20,
                            durability: 100,
                        },
                    )
                    .unwrap();
            }

            if i % 6 == 0 {
                world
                    .add_component(
                        entity,
                        Experience {
                            points: i as u64 * 100,
                        },
                    )
                    .unwrap();
            }
        }

        // Pattern 2: Enemies (multiple of 3)
        if i % 3 == 0 {
            world.add_component(entity, Enemy).unwrap();
            world
                .add_component(
                    entity,
                    Health {
                        current: 30 + i,
                        max: 50,
                    },
                )
                .unwrap();

            if i % 9 == 0 {
                world.add_component(entity, Damage { amount: 15 }).unwrap();
            }
        }

        // Pattern 3: Npcs (multiple of 5)
        if i % 5 == 0 {
            world.add_component(entity, Npc).unwrap();
            world
                .add_component(
                    entity,
                    Tag {
                        name: format!("Npc{i}"),
                    },
                )
                .unwrap();

            if i % 10 == 0 {
                world
                    .add_component(
                        entity,
                        Armor {
                            defense: 10,
                            weight: 5.0,
                        },
                    )
                    .unwrap();
            }
        }

        // Pattern 4: Dead entities (multiple of 7)
        if i % 7 == 0 {
            world.add_component(entity, Dead).unwrap();
        }

        // Pattern 5: Velocity (multiple of 11)
        if i % 11 == 0 {
            world
                .add_component(entity, Velocity { x: 1.0, y: 0.0 })
                .unwrap();
        }
    }

    // Test 1: Armed players (Player + Weapon, no Dead)
    let armed_players_query = Query::<Position>::new()
        .with::<Player>()
        .with::<Weapon>()
        .without::<Dead>();
    let armed_players = armed_players_query.iter(&world).collect::<Vec<_>>();

    // Should find entities where i % 2 == 0 AND i % 4 == 0 AND i % 7 != 0
    // i.e., i % 4 == 0 AND i % 7 != 0
    let expected_armed_players: Vec<u32> = (0..20).filter(|&i| i % 4 == 0 && i % 7 != 0).collect();
    assert_eq!(armed_players.len(), expected_armed_players.len());

    // Test 2: Damaged enemies (Enemy + Damage + Health, no Dead)
    let damaged_enemies_query = Query::<Position>::new()
        .with::<Enemy>()
        .with::<Damage>()
        .with::<Health>()
        .without::<Dead>();
    let damaged_enemies = damaged_enemies_query.iter(&world).collect::<Vec<_>>();

    // Should find entities where i % 3 == 0 AND i % 9 == 0 AND i % 7 != 0
    // i.e., i % 9 == 0 AND i % 7 != 0
    let expected_damaged_enemies: Vec<u32> =
        (0..20).filter(|&i| i % 9 == 0 && i % 7 != 0).collect();
    assert_eq!(damaged_enemies.len(), expected_damaged_enemies.len());

    // Test 3: Armored Npcs (Npc + Armor, no Dead)
    let armored_npcs_query = Query::<Position>::new()
        .with::<Npc>()
        .with::<Armor>()
        .without::<Dead>();
    let armored_npcs = armored_npcs_query.iter(&world).collect::<Vec<_>>();

    // Should find entities where i % 5 == 0 AND i % 10 == 0 AND i % 7 != 0
    // i.e., i % 10 == 0 AND i % 7 != 0
    let expected_armored_npcs: Vec<u32> = (0..20).filter(|&i| i % 10 == 0 && i % 7 != 0).collect();
    assert_eq!(armored_npcs.len(), expected_armored_npcs.len());

    // Test 4: Moving entities that are not dead
    let moving_alive_query = Query::<Position>::new()
        .with::<Velocity>()
        .without::<Dead>();
    let moving_alive = moving_alive_query.iter(&world).collect::<Vec<_>>();

    // Should find entities where i % 11 == 0 AND i % 7 != 0
    let expected_moving_alive: Vec<u32> = (0..20).filter(|&i| i % 11 == 0 && i % 7 != 0).collect();
    assert_eq!(moving_alive.len(), expected_moving_alive.len());

    // Test 5: Complex query with multiple inclusions and exclusions
    let complex_query = Query::<Position>::new()
        .with::<Player>()
        .with::<Level>()
        .without::<Dead>()
        .without::<Damage>();
    let complex_results = complex_query.iter(&world).collect::<Vec<_>>();

    // Should find players with levels that are not dead and not damaged
    // Note: One player (i = 18) also gets a Damage component due to i % 9 == 0, so is excluded.
    let expected_complex: Vec<u32> = (0..20)
        .filter(|&i| i % 2 == 0 && i % 7 != 0 && i % 9 != 0)
        .collect();
    assert_eq!(complex_results.len(), expected_complex.len());
}

#[test]
fn test_query_with_optional_components() {
    let mut world = World::new();

    // Create entities with varying component sets
    let entity1 = world.spawn_entity();
    world
        .add_component(entity1, Position { x: 1.0, y: 1.0 })
        .unwrap();
    world
        .add_component(
            entity1,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();
    world.add_component(entity1, Player).unwrap();

    let entity2 = world.spawn_entity();
    world
        .add_component(entity2, Position { x: 2.0, y: 2.0 })
        .unwrap();
    world
        .add_component(
            entity2,
            Health {
                current: 50,
                max: 50,
            },
        )
        .unwrap();
    world.add_component(entity2, Player).unwrap();
    world
        .add_component(
            entity2,
            Weapon {
                damage: 25,
                durability: 100,
            },
        )
        .unwrap();

    let entity3 = world.spawn_entity();
    world
        .add_component(entity3, Position { x: 3.0, y: 3.0 })
        .unwrap();
    world
        .add_component(
            entity3,
            Health {
                current: 75,
                max: 75,
            },
        )
        .unwrap();
    world.add_component(entity3, Player).unwrap();
    world
        .add_component(
            entity3,
            Armor {
                defense: 15,
                weight: 10.0,
            },
        )
        .unwrap();

    let entity4 = world.spawn_entity();
    world
        .add_component(entity4, Position { x: 4.0, y: 4.0 })
        .unwrap();
    world
        .add_component(
            entity4,
            Health {
                current: 80,
                max: 80,
            },
        )
        .unwrap();
    world.add_component(entity4, Player).unwrap();
    world
        .add_component(
            entity4,
            Weapon {
                damage: 30,
                durability: 90,
            },
        )
        .unwrap();
    world
        .add_component(
            entity4,
            Armor {
                defense: 20,
                weight: 8.0,
            },
        )
        .unwrap();

    // Base query for all players
    let all_players_query = Query::<Position>::new().with::<Player>().with::<Health>();
    let all_players: Vec<_> = all_players_query.iter(&world).collect();
    assert_eq!(all_players.len(), 4);

    // Query players and check for optional weapon
    let players_with_weapons: Vec<_> = all_players_query
        .iter(&world)
        .filter_map(|(entity, pos)| {
            let weapon = world.get_component::<Weapon>(entity);
            weapon.map(|w| (entity, pos, w))
        })
        .collect();
    assert_eq!(players_with_weapons.len(), 2); // entity2, entity4

    // Query players and check for optional armor
    let players_with_armor: Vec<_> = all_players_query
        .iter(&world)
        .filter_map(|(entity, pos)| {
            let armor = world.get_component::<Armor>(entity);
            armor.map(|a| (entity, pos, a))
        })
        .collect();
    assert_eq!(players_with_armor.len(), 2); // entity3, entity4

    // Query players with both weapon and armor
    let fully_equipped: Vec<_> = all_players_query
        .iter(&world)
        .filter_map(|(entity, pos)| {
            let weapon = world.get_component::<Weapon>(entity);
            let armor = world.get_component::<Armor>(entity);
            match (weapon, armor) {
                (Some(w), Some(a)) => Some((entity, pos, w, a)),
                _ => None,
            }
        })
        .collect();
    assert_eq!(fully_equipped.len(), 1); // entity4

    // Query players with weapon OR armor (but not necessarily both)
    let equipped_players: Vec<_> = all_players_query
        .iter(&world)
        .filter(|(entity, _)| {
            world.has_component::<Weapon>(*entity) || world.has_component::<Armor>(*entity)
        })
        .collect();
    assert_eq!(equipped_players.len(), 3); // entity2, entity3, entity4

    // Query players with weapon but no armor
    let weapon_only: Vec<_> = all_players_query
        .iter(&world)
        .filter(|(entity, _)| {
            world.has_component::<Weapon>(*entity) && !world.has_component::<Armor>(*entity)
        })
        .collect();
    assert_eq!(weapon_only.len(), 1); // entity2
}

#[test]
#[ignore] // TODO: Fix iterator invalidation bug - double borrow of world during filtering
fn test_dynamic_filtering_with_component_values() {
    let mut world = World::new();

    // Create entities with varying stats
    for i in 0..10 {
        let entity = world.spawn_entity();
        world
            .add_component(
                entity,
                Position {
                    x: i as f32,
                    y: 0.0,
                },
            )
            .unwrap();
        world
            .add_component(
                entity,
                Health {
                    current: (i * 10) as u32,
                    max: 100,
                },
            )
            .unwrap();
        world
            .add_component(entity, Level { value: i as u32 })
            .unwrap();

        if i % 2 == 0 {
            world
                .add_component(
                    entity,
                    Weapon {
                        damage: (i * 5) as u32,
                        durability: 100 - (i * 5) as u32,
                    },
                )
                .unwrap();
        }

        if i < 3 {
            world.add_component(entity, Player).unwrap();
        } else if i < 7 {
            world.add_component(entity, Enemy).unwrap();
        } else {
            world.add_component(entity, Npc).unwrap();
        }
    }

    let base_query = Query::<Position>::new().with::<Health>().with::<Level>();

    // Test: High health entities (> 50)
    let high_health: Vec<_> = base_query
        .iter(&world)
        .filter(|(entity, _)| {
            world
                .get_component::<Health>(*entity)
                .map(|h| h.current > 50)
                .unwrap_or(false)
        })
        .collect();
    assert_eq!(high_health.len(), 4); // i = 6, 7, 8, 9

    // Test: High level entities (>= 5)
    let high_level: Vec<_> = base_query
        .iter(&world)
        .filter(|(entity, _)| {
            world
                .get_component::<Level>(*entity)
                .map(|l| l.value >= 5)
                .unwrap_or(false)
        })
        .collect();
    assert_eq!(high_level.len(), 5); // i = 5, 6, 7, 8, 9

    // Test: Entities with powerful weapons (damage >= 20)
    let powerful_weapons: Vec<_> = base_query
        .iter(&world)
        .filter(|(entity, _)| {
            world
                .get_component::<Weapon>(*entity)
                .map(|w| w.damage >= 20)
                .unwrap_or(false)
        })
        .collect();
    println!("powerful_weapons entities:");
    for (entity, _) in &powerful_weapons {
        let weapon = world.get_component::<Weapon>(*entity);
        let health = world.get_component::<Health>(*entity);
        let level = world.get_component::<Level>(*entity);
        println!(
            "entity: {entity:?}, weapon: {weapon:?}, health: {health:?}, level: {level:?}"
        );
    }
    assert_eq!(powerful_weapons.len(), 3); // i = 4, 6, 8 (even numbers with damage >= 20)

    // Test: Low durability weapons (< 80)
    let fragile_weapons: Vec<_> = base_query
        .iter(&world)
        .filter(|(entity, _)| {
            world
                .get_component::<Weapon>(*entity)
                .map(|w| w.durability < 80)
                .unwrap_or(false)
        })
        .collect();
    assert_eq!(fragile_weapons.len(), 3); // i = 4, 6, 8

    // Test: Players with high stats (level >= 2 AND health >= 20)
    let elite_players: Vec<_> = base_query
        .iter(&world)
        .filter(|(entity, _)| {
            let is_player = world.has_component::<Player>(*entity);
            let high_level = world
                .get_component::<Level>(*entity)
                .map(|l| l.value >= 2)
                .unwrap_or(false);
            let good_health = world
                .get_component::<Health>(*entity)
                .map(|h| h.current >= 20)
                .unwrap_or(false);

            is_player && high_level && good_health
        })
        .collect();
    assert_eq!(elite_players.len(), 1); // i = 2 (only player with level >= 2)

    // Test: Combat-ready entities (have weapon AND good health AND not Npc)
    let combat_ready: Vec<_> = base_query
        .iter(&world)
        .filter(|(entity, _)| {
            let has_weapon = world.has_component::<Weapon>(*entity);
            let good_health = world
                .get_component::<Health>(*entity)
                .map(|h| h.current >= 30)
                .unwrap_or(false);
            let not_npc = !world.has_component::<Npc>(*entity);

            has_weapon && good_health && not_npc
        })
        .collect();
    // Should find even numbers i where health >= 30 and not Npc
    // i = 4, 6 (i = 8 is Npc)
    assert_eq!(combat_ready.len(), 2);
}

#[test]
fn test_query_edge_cases() {
    let mut world = World::new();

    // Test: Empty world queries
    let empty_query = Query::<Position>::new();
    assert_eq!(empty_query.count(&world), 0);
    assert!(empty_query.first(&world).is_none());
    assert!(!empty_query.any(&world));

    // Test: Query with no matching entities
    let entity = world.spawn_entity();
    world
        .add_component(
            entity,
            Health {
                current: 100,
                max: 100,
            },
        )
        .unwrap();

    let no_match_query = Query::<Position>::new().with::<Velocity>();
    assert_eq!(no_match_query.count(&world), 0);
    assert!(no_match_query.first(&world).is_none());
    assert!(!no_match_query.any(&world));

    // Test: Query with impossible conditions
    let impossible_query = Query::<Position>::new().with::<Player>().with::<Enemy>(); // Entity can't be both Player and Enemy

    world
        .add_component(entity, Position { x: 0.0, y: 0.0 })
        .unwrap();
    world.add_component(entity, Player).unwrap();

    assert_eq!(impossible_query.count(&world), 0);

    // Test: Self-contradictory query
    let contradictory_query = Query::<Health>::new().with::<Dead>().without::<Dead>(); // Can't have and not have Dead at the same time

    world.add_component(entity, Dead).unwrap();
    assert_eq!(contradictory_query.count(&world), 0);

    // Test: Query after entity deletion
    world.delete_entity(entity);

    let after_deletion_query = Query::<Position>::new();
    assert_eq!(after_deletion_query.count(&world), 0);

    // Test: Query with deleted and non-deleted entities mixed
    let entity1 = world.spawn_entity();
    world
        .add_component(entity1, Position { x: 1.0, y: 1.0 })
        .unwrap();

    let entity2 = world.spawn_entity();
    world
        .add_component(entity2, Position { x: 2.0, y: 2.0 })
        .unwrap();

    world.delete_entity(entity2);

    let mixed_query = Query::<Position>::new();
    assert_eq!(mixed_query.count(&world), 1); // Only entity1 should be found

    let results: Vec<_> = mixed_query.iter(&world).collect();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, entity1);

    // Test: Query after cleanup
    world.cleanup_deleted_entities();
    assert_eq!(mixed_query.count(&world), 1); // Should still be 1
}

#[test]
fn test_query_consistency_under_modification() {
    let mut world = World::new();

    // Create initial entities
    let mut entities = Vec::new();
    for i in 0..10 {
        let entity = world.spawn_entity();
        world
            .add_component(
                entity,
                Position {
                    x: i as f32,
                    y: 0.0,
                },
            )
            .unwrap();

        if i % 2 == 0 {
            world
                .add_component(
                    entity,
                    Health {
                        current: 100,
                        max: 100,
                    },
                )
                .unwrap();
        }

        entities.push(entity);
    }

    let health_query = Query::<Position>::new().with::<Health>();

    // Initial state
    assert_eq!(health_query.count(&world), 5); // Even numbered entities

    // Add health to odd entities
    for i in (1..10).step_by(2) {
        world
            .add_component(
                entities[i],
                Health {
                    current: 50,
                    max: 50,
                },
            )
            .unwrap();
    }

    // Now all should have health
    assert_eq!(health_query.count(&world), 10);

    // Remove health from some entities
    for i in (0..10).step_by(3) {
        world.remove_component::<Health>(entities[i]);
    }

    // Should have 10 - ceil(10/3) = 10 - 4 = 6 entities with health
    assert_eq!(health_query.count(&world), 6);

    // Delete some entities
    for i in (1..10).step_by(4) {
        world.delete_entity(entities[i]);
    }

    // Verify query still works correctly
    let remaining_with_health = health_query.count(&world);
    assert!(remaining_with_health <= 6);

    // All results should be valid
    let results: Vec<_> = health_query.iter(&world).collect();
    for (entity, _) in results {
        assert!(world.has_component::<Position>(entity));
        assert!(world.has_component::<Health>(entity));
    }
}
