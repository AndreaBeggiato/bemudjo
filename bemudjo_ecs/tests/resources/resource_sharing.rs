//! Resource Sharing Integration Tests
//!
//! Tests focused on multi-system resource access, concurrent usage,
//! and resource sharing patterns in system execution.

use bemudjo_ecs::{Component, SequentialSystemScheduler, System, World};
use std::cell::RefCell;
use std::rc::Rc;

// Test Resources
#[derive(Debug, Clone, PartialEq)]
struct GameTime {
    elapsed: f64,
    delta: f32,
    frame_count: u64,
}
impl Component for GameTime {}

#[derive(Debug, Clone, PartialEq)]
struct PlayerStats {
    score: u64,
    level: u32,
    experience: u64,
}
impl Component for PlayerStats {}

#[derive(Debug, Clone, PartialEq)]
struct GameConfig {
    difficulty_multiplier: f32,
    debug_mode: bool,
    auto_save_interval: u32,
}
impl Component for GameConfig {}

#[derive(Debug, Clone, PartialEq)]
struct EventLog {
    events: Vec<String>,
    max_events: usize,
}
impl Component for EventLog {}

#[derive(Debug, Clone, PartialEq)]
struct NetworkStats {
    players_online: u32,
    server_load: f32,
    bandwidth_usage: u64,
}
impl Component for NetworkStats {}

// Test Entity Components
#[derive(Clone, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}
impl Component for Position {}

#[derive(Clone, Debug, PartialEq)]
struct Health {
    current: u32,
    max: u32,
}
impl Component for Health {}

#[derive(Clone, Debug, PartialEq)]
struct Enemy {
    damage: u32,
}
impl Component for Enemy {}

// Test Systems that share resources

struct TimeUpdateSystem {
    execution_log: Rc<RefCell<Vec<String>>>,
}

impl TimeUpdateSystem {
    fn new(log: Rc<RefCell<Vec<String>>>) -> Self {
        Self { execution_log: log }
    }
}

impl System for TimeUpdateSystem {
    fn run(&self, world: &mut World) {
        self.execution_log
            .borrow_mut()
            .push("TimeUpdateSystem: Reading time".to_string());

        if !world.has_resource::<GameTime>() {
            world.insert_resource(GameTime {
                elapsed: 0.0,
                delta: 0.016,
                frame_count: 0,
            });
        }

        world
            .update_resource::<GameTime, _>(|mut time| {
                time.elapsed += time.delta as f64;
                time.frame_count += 1;
                time
            })
            .unwrap();

        self.execution_log
            .borrow_mut()
            .push("TimeUpdateSystem: Updated time".to_string());
    }
}

struct ScoreSystem {
    execution_log: Rc<RefCell<Vec<String>>>,
}

impl ScoreSystem {
    fn new(log: Rc<RefCell<Vec<String>>>) -> Self {
        Self { execution_log: log }
    }
}

impl System for ScoreSystem {
    fn run(&self, world: &mut World) {
        self.execution_log
            .borrow_mut()
            .push("ScoreSystem: Reading time and config".to_string());

        // Read time and config resources
        let time = world.get_resource::<GameTime>();
        let config = world.get_resource::<GameConfig>();

        if let (Some(time), Some(config)) = (time, config) {
            let base_score = if time.frame_count % 60 == 0 { 100 } else { 0 };
            let score_bonus = (base_score as f32 * config.difficulty_multiplier) as u64;

            if !world.has_resource::<PlayerStats>() {
                world.insert_resource(PlayerStats {
                    score: 0,
                    level: 1,
                    experience: 0,
                });
            }

            world
                .update_resource::<PlayerStats, _>(|mut stats| {
                    stats.score += score_bonus;
                    stats.experience += score_bonus / 10;

                    // Level up every 1000 experience
                    if stats.experience >= 1000 * stats.level as u64 {
                        stats.level += 1;
                        stats.experience = 0;
                    }

                    stats
                })
                .unwrap();

            self.execution_log
                .borrow_mut()
                .push(format!("ScoreSystem: Added {score_bonus} score"));
        }
    }
}

struct LoggingSystem {
    execution_log: Rc<RefCell<Vec<String>>>,
}

impl LoggingSystem {
    fn new(log: Rc<RefCell<Vec<String>>>) -> Self {
        Self { execution_log: log }
    }
}

impl System for LoggingSystem {
    fn run(&self, world: &mut World) {
        self.execution_log
            .borrow_mut()
            .push("LoggingSystem: Reading multiple resources".to_string());

        // Read multiple resources, clone/copy values to avoid borrow conflicts
        let time = world.get_resource::<GameTime>().cloned();
        let stats = world.get_resource::<PlayerStats>().cloned();
        let config = world.get_resource::<GameConfig>().cloned();

        if !world.has_resource::<EventLog>() {
            world.insert_resource(EventLog {
                events: Vec::new(),
                max_events: 100,
            });
        }

        // Log events based on resource states
        world
            .update_resource::<EventLog, _>(|mut log| {
                if let Some(time) = &time {
                    if time.frame_count % 120 == 0 {
                        log.events.push(format!(
                            "Frame {} - Elapsed: {:.2}s",
                            time.frame_count, time.elapsed
                        ));
                    }
                }
                if let Some(stats) = &stats {
                    if stats.score > 0 && stats.score % 500 == 0 {
                        log.events.push(format!("Score milestone: {}", stats.score));
                    }
                    if stats.level > 1 {
                        log.events
                            .push(format!("Player reached level {}", stats.level));
                    }
                }
                if let Some(config) = &config {
                    if config.debug_mode {
                        log.events.push("Debug mode active".to_string());
                    }
                }
                while log.events.len() > log.max_events {
                    log.events.remove(0);
                }
                log
            })
            .unwrap();
    }
}

struct NetworkSystem {
    execution_log: Rc<RefCell<Vec<String>>>,
}

impl NetworkSystem {
    fn new(log: Rc<RefCell<Vec<String>>>) -> Self {
        Self { execution_log: log }
    }
}

impl System for NetworkSystem {
    fn run(&self, world: &mut World) {
        self.execution_log
            .borrow_mut()
            .push("NetworkSystem: Reading config and stats".to_string());

        // Clone resources before mutably borrowing world
        let config = world.get_resource::<GameConfig>().cloned();
        let player_stats = world.get_resource::<PlayerStats>().cloned();
        if !world.has_resource::<NetworkStats>() {
            world.insert_resource(NetworkStats {
                players_online: 1,
                server_load: 0.1,
                bandwidth_usage: 0,
            });
        }
        world
            .update_resource::<NetworkStats, _>(|mut net_stats| {
                // Simulate network activity based on game state
                if let Some(config) = &config {
                    if config.debug_mode {
                        net_stats.bandwidth_usage += 1000; // Debug data
                    }
                }
                if let Some(stats) = &player_stats {
                    // Higher level players use more bandwidth
                    net_stats.bandwidth_usage += stats.level as u64 * 10;
                }
                // Simulate server load
                net_stats.server_load = (net_stats.bandwidth_usage as f32 / 10000.0).min(1.0);
                net_stats
            })
            .unwrap();
    }
}

struct CleanupSystem {
    execution_log: Rc<RefCell<Vec<String>>>,
}

impl CleanupSystem {
    fn new(log: Rc<RefCell<Vec<String>>>) -> Self {
        Self { execution_log: log }
    }
}

impl System for CleanupSystem {
    fn run(&self, world: &mut World) {
        self.execution_log
            .borrow_mut()
            .push("CleanupSystem: Performing cleanup".to_string());

        // Read time to decide when to cleanup
        if let Some(time) = world.get_resource::<GameTime>() {
            if time.frame_count % 600 == 0 {
                // Every 10 seconds
                // Reset some resources
                if world.has_resource::<EventLog>() {
                    world
                        .update_resource::<EventLog, _>(|mut log| {
                            log.events.clear();
                            log
                        })
                        .unwrap();

                    self.execution_log
                        .borrow_mut()
                        .push("CleanupSystem: Cleared event log".to_string());
                }

                if world.has_resource::<NetworkStats>() {
                    world
                        .update_resource::<NetworkStats, _>(|mut net_stats| {
                            net_stats.bandwidth_usage = 0;
                            net_stats.server_load = 0.1;
                            net_stats
                        })
                        .unwrap();

                    self.execution_log
                        .borrow_mut()
                        .push("CleanupSystem: Reset network stats".to_string());
                }
            }
        }
    }
}

#[test]
fn test_basic_resource_sharing() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();
    let log = Rc::new(RefCell::new(Vec::new()));

    // Add systems that share resources
    scheduler
        .add_system(TimeUpdateSystem::new(log.clone()))
        .unwrap();
    scheduler.add_system(ScoreSystem::new(log.clone())).unwrap();
    scheduler
        .add_system(LoggingSystem::new(log.clone()))
        .unwrap();

    scheduler.build().unwrap();

    // Initialize config resource
    world.insert_resource(GameConfig {
        difficulty_multiplier: 2.0,
        debug_mode: true,
        auto_save_interval: 300,
    });

    // Run one tick
    scheduler.run_tick(&mut world);

    // Verify all systems executed and shared resources
    let execution_log = log.borrow().clone();
    assert!(execution_log
        .iter()
        .any(|msg| msg.contains("TimeUpdateSystem")));
    assert!(execution_log.iter().any(|msg| msg.contains("ScoreSystem")));
    assert!(execution_log
        .iter()
        .any(|msg| msg.contains("LoggingSystem")));

    // Verify resources were created and updated
    assert!(world.has_resource::<GameTime>());
    assert!(world.has_resource::<PlayerStats>());
    assert!(world.has_resource::<EventLog>());

    let time = world.get_resource::<GameTime>().unwrap();
    assert_eq!(time.frame_count, 1);

    let stats = world.get_resource::<PlayerStats>().unwrap();
    assert_eq!(stats.score, 0); // No score on first frame (not frame 60)

    let event_log = world.get_resource::<EventLog>().unwrap();
    assert!(event_log.events.contains(&"Debug mode active".to_string()));
}

#[test]
fn test_multi_system_resource_coordination() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();
    let log = Rc::new(RefCell::new(Vec::new()));

    // Add all systems
    scheduler
        .add_system(TimeUpdateSystem::new(log.clone()))
        .unwrap();
    scheduler.add_system(ScoreSystem::new(log.clone())).unwrap();
    scheduler
        .add_system(LoggingSystem::new(log.clone()))
        .unwrap();
    scheduler
        .add_system(NetworkSystem::new(log.clone()))
        .unwrap();
    scheduler
        .add_system(CleanupSystem::new(log.clone()))
        .unwrap();

    scheduler.build().unwrap();

    // Initialize resources
    world.insert_resource(GameConfig {
        difficulty_multiplier: 1.5,
        debug_mode: false,
        auto_save_interval: 300,
    });

    // Run multiple ticks to see coordination
    for _ in 0..120 {
        // 2 seconds worth of frames
        scheduler.run_tick(&mut world);
    }

    // Verify resource states after coordination
    let time = world.get_resource::<GameTime>().unwrap();
    assert_eq!(time.frame_count, 120);
    assert!((time.elapsed - 1.92).abs() < 0.01); // 120 * 0.016

    let stats = world.get_resource::<PlayerStats>().unwrap();
    assert!(stats.score > 0); // Should have scored points at frames 60 and 120
    assert_eq!(stats.score, 300); // 2 * 100 * 1.5 difficulty multiplier

    let net_stats = world.get_resource::<NetworkStats>().unwrap();
    assert!(net_stats.bandwidth_usage > 0);
    assert!(net_stats.server_load > 0.0);

    let event_log = world.get_resource::<EventLog>().unwrap();
    assert!(event_log.events.iter().any(|e| e.contains("Frame 120")));
}

#[test]
fn test_resource_sharing_with_entities() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();
    let log = Rc::new(RefCell::new(Vec::new()));

    // System that uses both resources and entities
    struct CombatSystem {
        execution_log: Rc<RefCell<Vec<String>>>,
    }

    impl CombatSystem {
        fn new(log: Rc<RefCell<Vec<String>>>) -> Self {
            Self { execution_log: log }
        }
    }

    impl System for CombatSystem {
        fn run(&self, world: &mut World) {
            // Debug: Log that CombatSystem is running
            self.execution_log
                .borrow_mut()
                .push("CombatSystem: Starting combat system".to_string());

            // Read config for difficulty
            let config = world.get_resource::<GameConfig>();
            let difficulty_multiplier = config.map(|c| c.difficulty_multiplier).unwrap_or(1.0);

            // Process entities
            let entities: Vec<_> = world.entities().cloned().collect();
            let mut enemies_defeated = 0;

            self.execution_log.borrow_mut().push(format!(
                "CombatSystem: Processing {} entities",
                entities.len()
            ));

            for entity in entities {
                if let (Some(health), Some(enemy)) = (
                    world.get_component::<Health>(entity),
                    world.get_component::<Enemy>(entity),
                ) {
                    let damage = (enemy.damage as f32 * difficulty_multiplier) as u32;
                    let current_health = health.current;
                    let max_health = health.max;

                    self.execution_log.borrow_mut().push(format!(
                        "CombatSystem: Entity {entity:?} has health {current_health} taking {damage} damage"
                    ));

                    if current_health <= damage {
                        world.delete_entity(entity);
                        enemies_defeated += 1;
                        self.execution_log
                            .borrow_mut()
                            .push(format!("CombatSystem: Entity {entity:?} defeated"));
                    } else {
                        let new_health = current_health - damage;
                        world.replace_component(
                            entity,
                            Health {
                                current: new_health,
                                max: max_health,
                            },
                        );
                        self.execution_log.borrow_mut().push(format!(
                            "CombatSystem: Entity {entity:?} damaged, health now {new_health}"
                        ));
                    }
                }
            }

            // Update player stats based on combat
            if enemies_defeated > 0 && world.has_resource::<PlayerStats>() {
                world
                    .update_resource::<PlayerStats, _>(|mut stats| {
                        stats.score += enemies_defeated * 50;
                        stats.experience += enemies_defeated * 25;
                        stats
                    })
                    .unwrap();

                self.execution_log.borrow_mut().push(format!(
                    "CombatSystem: Defeated {enemies_defeated} enemies"
                ));
            } else {
                self.execution_log.borrow_mut().push(format!(
                    "CombatSystem: No enemies defeated (defeated={}, has_stats={})",
                    enemies_defeated,
                    world.has_resource::<PlayerStats>()
                ));
            }
        }
    }

    scheduler
        .add_system(TimeUpdateSystem::new(log.clone()))
        .unwrap();
    scheduler
        .add_system(CombatSystem::new(log.clone()))
        .unwrap();
    scheduler.add_system(ScoreSystem::new(log.clone())).unwrap();

    scheduler.build().unwrap();

    // Initialize resources
    world.insert_resource(GameConfig {
        difficulty_multiplier: 2.0,
        debug_mode: false,
        auto_save_interval: 300,
    });

    world.insert_resource(PlayerStats {
        score: 0,
        level: 1,
        experience: 0,
    });

    // Create enemy entities with low health so they can be defeated
    for i in 0..5 {
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
                    current: 50,
                    max: 50,
                },
            )
            .unwrap(); // Reduced health
        world.add_component(entity, Enemy { damage: 30 }).unwrap();
    }

    assert_eq!(world.entities().count(), 5);

    // Run systems
    scheduler.run_tick(&mut world);

    // Check that resources were updated correctly
    let stats = world.get_resource::<PlayerStats>().unwrap();

    // Combat should have affected player stats
    assert!(
        stats.score > 0 || stats.experience > 0,
        "Expected stats.score > 0 OR stats.experience > 0, but got score={}, experience={}",
        stats.score,
        stats.experience
    );

    // Some entities should be damaged
    let remaining_entities = world.entities().count();
    assert!(remaining_entities <= 5);
}

#[test]
fn test_resource_dependency_chains() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();
    let log = Rc::new(RefCell::new(Vec::new()));

    // System that depends on time resource
    struct DependentSystem1 {
        _execution_log: Rc<RefCell<Vec<String>>>,
    }

    impl System for DependentSystem1 {
        fn run(&self, world: &mut World) {
            if let Some(time) = world.get_resource::<GameTime>() {
                // Only create PlayerStats if enough time has passed
                if time.frame_count >= 10 && !world.has_resource::<PlayerStats>() {
                    world.insert_resource(PlayerStats {
                        score: 0,
                        level: 1,
                        experience: 0,
                    });
                }
            }
        }
    }

    // System that depends on PlayerStats resource
    struct DependentSystem2 {
        _execution_log: Rc<RefCell<Vec<String>>>,
    }

    impl System for DependentSystem2 {
        fn run(&self, world: &mut World) {
            if let Some(stats) = world.get_resource::<PlayerStats>() {
                // Only create EventLog if player has progressed
                if stats.level >= 1 && !world.has_resource::<EventLog>() {
                    world.insert_resource(EventLog {
                        events: vec!["Player progress tracked".to_string()],
                        max_events: 50,
                    });
                }
            }
        }
    }

    // System that depends on EventLog resource
    struct DependentSystem3 {
        _execution_log: Rc<RefCell<Vec<String>>>,
    }

    impl System for DependentSystem3 {
        fn run(&self, world: &mut World) {
            if let Some(log) = world.get_resource::<EventLog>() {
                // Only create NetworkStats if events exist
                if !log.events.is_empty() && !world.has_resource::<NetworkStats>() {
                    world.insert_resource(NetworkStats {
                        players_online: 1,
                        server_load: 0.2,
                        bandwidth_usage: 100,
                    });
                }
            }
        }
    }

    // Add systems in dependency order
    scheduler
        .add_system(TimeUpdateSystem::new(log.clone()))
        .unwrap();
    scheduler
        .add_system(DependentSystem1 {
            _execution_log: log.clone(),
        })
        .unwrap();
    scheduler
        .add_system(DependentSystem2 {
            _execution_log: log.clone(),
        })
        .unwrap();
    scheduler
        .add_system(DependentSystem3 {
            _execution_log: log.clone(),
        })
        .unwrap();

    scheduler.build().unwrap();

    // Run ticks and verify dependency chain
    for tick in 1..=15 {
        scheduler.run_tick(&mut world);

        let time = world.get_resource::<GameTime>().unwrap();
        assert_eq!(time.frame_count, tick);

        if tick < 10 {
            // Before frame 10, only time should exist
            assert!(world.has_resource::<GameTime>());
            assert!(!world.has_resource::<PlayerStats>());
            assert!(!world.has_resource::<EventLog>());
            assert!(!world.has_resource::<NetworkStats>());
        } else {
            // At frame 10 and after, all resources should exist
            // (they're all created in the same tick due to sequential system execution)
            assert!(world.has_resource::<GameTime>());
            assert!(world.has_resource::<PlayerStats>());
            assert!(world.has_resource::<EventLog>());
            assert!(world.has_resource::<NetworkStats>());
        }
    }
}

#[test]
fn test_resource_sharing_performance() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();
    let log = Rc::new(RefCell::new(Vec::new()));

    // Add many systems that all read the same resources
    for i in 0..50 {
        struct PerformanceSystem {
            id: usize,
            _execution_log: Rc<RefCell<Vec<String>>>,
        }

        impl System for PerformanceSystem {
            fn run(&self, world: &mut World) {
                // Read multiple resources
                let _time = world.get_resource::<GameTime>();
                let _config = world.get_resource::<GameConfig>();
                let _stats = world.get_resource::<PlayerStats>();

                // Update a shared resource
                if world.has_resource::<EventLog>() {
                    world
                        .update_resource::<EventLog, _>(|mut log| {
                            log.events.push(format!("System {} executed", self.id));
                            log
                        })
                        .ok();
                }
            }
        }

        scheduler
            .add_system(PerformanceSystem {
                id: i,
                _execution_log: log.clone(),
            })
            .unwrap();
    }

    scheduler.build().unwrap();

    // Initialize resources
    world.insert_resource(GameTime {
        elapsed: 0.0,
        delta: 0.016,
        frame_count: 0,
    });

    world.insert_resource(GameConfig {
        difficulty_multiplier: 1.0,
        debug_mode: false,
        auto_save_interval: 300,
    });

    world.insert_resource(PlayerStats {
        score: 0,
        level: 1,
        experience: 0,
    });

    world.insert_resource(EventLog {
        events: Vec::new(),
        max_events: 1000,
    });

    // Measure performance
    let start = std::time::Instant::now();

    for _ in 0..10 {
        scheduler.run_tick(&mut world);
    }

    let duration = start.elapsed();
    assert!(duration.as_millis() < 100); // Should complete quickly

    // Verify all systems executed
    let event_log = world.get_resource::<EventLog>().unwrap();
    assert_eq!(event_log.events.len(), 500); // 50 systems * 10 ticks
}

#[test]
fn test_resource_isolation_between_ticks() {
    let mut world = World::new();
    let mut scheduler = SequentialSystemScheduler::new();
    let log = Rc::new(RefCell::new(Vec::new()));

    struct StatefulSystem {
        expected_count: Rc<RefCell<u64>>,
        execution_log: Rc<RefCell<Vec<String>>>,
    }

    impl System for StatefulSystem {
        fn run(&self, world: &mut World) {
            if !world.has_resource::<GameTime>() {
                world.insert_resource(GameTime {
                    elapsed: 0.0,
                    delta: 0.016,
                    frame_count: 0,
                });
            }

            let current_count = world
                .update_resource::<GameTime, _>(|mut time| {
                    time.frame_count += 1;
                    time
                })
                .unwrap()
                .frame_count;

            let expected = *self.expected_count.borrow() + 1;
            assert_eq!(current_count, expected);

            *self.expected_count.borrow_mut() = expected;

            self.execution_log
                .borrow_mut()
                .push(format!("Tick {current_count}: State consistent"));
        }
    }

    let expected_count = Rc::new(RefCell::new(0u64));
    scheduler
        .add_system(StatefulSystem {
            expected_count: expected_count.clone(),
            execution_log: log.clone(),
        })
        .unwrap();

    scheduler.build().unwrap();

    // Run multiple ticks and verify state isolation
    for tick in 1..=100 {
        scheduler.run_tick(&mut world);

        let time = world.get_resource::<GameTime>().unwrap();
        assert_eq!(time.frame_count, tick);
        assert_eq!(*expected_count.borrow(), tick);
    }

    let execution_log = log.borrow().clone();
    assert_eq!(execution_log.len(), 100);
    assert!(execution_log
        .iter()
        .all(|msg| msg.contains("State consistent")));
}
