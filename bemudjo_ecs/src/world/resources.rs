use crate::{Component, ComponentError, ComponentStorage};

use super::World;

impl World {
    /// Inserts or replaces a global resource.
    ///
    /// Resources are global singleton data that can be accessed by all systems.
    /// Unlike entity components, resources can be replaced multiple times without error.
    ///
    /// # Parameters
    /// * `resource` - The resource instance to store globally
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct GameTime { delta: f32 }
    /// impl Component for GameTime {}
    ///
    /// let mut world = World::new();
    /// world.insert_resource(GameTime { delta: 0.016 });
    /// world.insert_resource(GameTime { delta: 0.033 }); // Replaces previous
    /// ```
    pub fn insert_resource<T: Component>(&mut self, resource: T) {
        let resource_entity = self.resource_entity;
        let storage = self.get_storage_mut::<T>();
        storage.insert_or_update(resource_entity, resource);
    }

    /// Gets an immutable reference to a global resource.
    ///
    /// Returns `None` if the resource doesn't exist or hasn't been inserted.
    ///
    /// # Returns
    /// * `Some(&T)` if the resource exists
    /// * `None` if the resource doesn't exist
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct GameTime { delta: f32 }
    /// impl Component for GameTime {}
    ///
    /// let mut world = World::new();
    /// world.insert_resource(GameTime { delta: 0.016 });
    ///
    /// let time = world.get_resource::<GameTime>().unwrap();
    /// assert_eq!(time.delta, 0.016);
    ///
    /// // Returns None for non-existent resources
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Settings { volume: f32 }
    /// impl Component for Settings {}
    /// assert!(world.get_resource::<Settings>().is_none());
    /// ```
    pub fn get_resource<T: Component>(&self) -> Option<&T> {
        let resource_entity = self.resource_entity;
        let storage = self.get_storage::<T>();
        storage?.get(resource_entity)
    }

    /// Removes a global resource and returns its value.
    ///
    /// Returns `None` if the resource doesn't exist.
    ///
    /// # Returns
    /// * `Some(T)` - The resource value if it existed
    /// * `None` - If the resource didn't exist
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct GameSettings { volume: f32 }
    /// impl Component for GameSettings {}
    ///
    /// let mut world = World::new();
    /// world.insert_resource(GameSettings { volume: 0.8 });
    ///
    /// let removed = world.remove_resource::<GameSettings>().unwrap();
    /// assert_eq!(removed.volume, 0.8);
    ///
    /// // Resource no longer exists
    /// assert!(world.get_resource::<GameSettings>().is_none());
    /// ```
    pub fn remove_resource<T: Component>(&mut self) -> Option<T> {
        let resource_entity = self.resource_entity;
        let storage = self.get_storage_mut::<T>();
        storage.remove(resource_entity)
    }

    /// Checks if a global resource exists.
    ///
    /// Returns `true` if the resource has been inserted and hasn't been removed.
    ///
    /// # Returns
    /// * `true` if the resource exists
    /// * `false` if the resource doesn't exist or was never inserted
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct InputState { mouse_x: f32, mouse_y: f32 }
    /// impl Component for InputState {}
    ///
    /// let mut world = World::new();
    ///
    /// // Initially no resource exists
    /// assert!(!world.has_resource::<InputState>());
    ///
    /// // After insertion
    /// world.insert_resource(InputState { mouse_x: 100.0, mouse_y: 50.0 });
    /// assert!(world.has_resource::<InputState>());
    ///
    /// // After removal
    /// world.remove_resource::<InputState>();
    /// assert!(!world.has_resource::<InputState>());
    /// ```
    pub fn has_resource<T: Component>(&self) -> bool {
        let resource_entity = self.resource_entity;
        let storage = self.get_storage::<T>();
        storage.is_some_and(|s| s.contains(resource_entity))
    }

    /// Updates a global resource using a closure and returns the new value.
    ///
    /// This method retrieves the current resource value, applies the provided closure
    /// to transform it, stores the updated value, and returns the new value.
    ///
    /// # Parameters
    /// * `f` - A closure that takes the current resource value and returns the updated value
    ///
    /// # Returns
    /// * `Ok(T)` - The updated resource value
    /// * `Err(ComponentError::ComponentNotFound)` - If the resource doesn't exist
    ///
    /// # Type Parameters
    /// * `T` - The resource type, must implement `Component` and `Clone`
    /// * `F` - The closure type that transforms the resource
    ///
    /// # Example
    /// ```
    /// use bemudjo_ecs::{World, Component};
    ///
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Score { value: u32 }
    /// impl Component for Score {}
    ///
    /// let mut world = World::new();
    /// world.insert_resource(Score { value: 100 });
    ///
    /// // Update the score by adding 50
    /// let new_score = world.update_resource::<Score, _>(|mut score| {
    ///     score.value += 50;
    ///     score
    /// }).unwrap();
    ///
    /// assert_eq!(new_score.value, 150);
    /// assert_eq!(world.get_resource::<Score>().unwrap().value, 150);
    ///
    /// // Trying to update non-existent resource fails
    /// #[derive(Clone, Debug, PartialEq)]
    /// struct Settings { volume: f32 }
    /// impl Component for Settings {}
    /// let result = world.update_resource::<Settings, _>(|s| s);
    /// assert!(result.is_err());
    /// ```
    pub fn update_resource<T, F>(&mut self, f: F) -> Result<T, ComponentError>
    where
        T: Component + Clone,
        F: FnOnce(T) -> T,
    {
        let resource_entity = self.resource_entity;
        let storage = self.get_storage_mut::<T>();

        match storage.get(resource_entity).cloned() {
            Some(current) => {
                let updated = f(current);
                storage.insert_or_update(resource_entity, updated.clone());
                Ok(updated)
            }
            None => Err(ComponentError::ComponentNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Component, ComponentError};

    // Test helper structs
    #[derive(Debug, Clone, PartialEq)]
    struct GameTime {
        delta: f32,
        total: f32,
    }
    impl Component for GameTime {}

    #[derive(Debug, Clone, PartialEq)]
    struct PlayerScore {
        value: u32,
        high_score: u32,
    }
    impl Component for PlayerScore {}

    #[derive(Debug, Clone, PartialEq)]
    struct GameSettings {
        volume: f32,
        difficulty: u8,
    }
    impl Component for GameSettings {}

    #[derive(Debug, Clone, PartialEq)]
    struct InputState {
        mouse_x: f32,
        mouse_y: f32,
        keys_pressed: Vec<String>,
    }
    impl Component for InputState {}

    #[test]
    fn test_insert_resource() {
        let mut world = World::new();
        let game_time = GameTime {
            delta: 0.016,
            total: 120.5,
        };

        world.insert_resource(game_time.clone());

        assert!(world.has_resource::<GameTime>());
        assert_eq!(world.get_resource::<GameTime>().unwrap(), &game_time);
    }

    #[test]
    fn test_insert_resource_replace() {
        let mut world = World::new();

        // Insert initial resource
        let initial_time = GameTime {
            delta: 0.016,
            total: 100.0,
        };
        world.insert_resource(initial_time);

        // Replace with new resource
        let new_time = GameTime {
            delta: 0.033,
            total: 150.0,
        };
        world.insert_resource(new_time.clone());

        // Should have the new resource
        assert_eq!(world.get_resource::<GameTime>().unwrap(), &new_time);
    }

    #[test]
    fn test_get_resource_exists() {
        let mut world = World::new();
        let score = PlayerScore {
            value: 1500,
            high_score: 2000,
        };
        world.insert_resource(score.clone());

        let retrieved = world.get_resource::<PlayerScore>();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &score);
    }

    #[test]
    fn test_get_resource_not_exists() {
        let world = World::new();

        let result = world.get_resource::<GameTime>();
        assert!(result.is_none());
    }

    #[test]
    fn test_remove_resource_exists() {
        let mut world = World::new();
        let settings = GameSettings {
            volume: 0.8,
            difficulty: 3,
        };
        world.insert_resource(settings.clone());

        let removed = world.remove_resource::<GameSettings>();
        assert_eq!(removed, Some(settings));
        assert!(!world.has_resource::<GameSettings>());
    }

    #[test]
    fn test_remove_resource_not_exists() {
        let mut world = World::new();

        let removed = world.remove_resource::<GameTime>();
        assert_eq!(removed, None);
    }

    #[test]
    fn test_has_resource_exists() {
        let mut world = World::new();
        let input = InputState {
            mouse_x: 100.0,
            mouse_y: 50.0,
            keys_pressed: vec!["W".to_string(), "A".to_string()],
        };
        world.insert_resource(input);

        assert!(world.has_resource::<InputState>());
    }

    #[test]
    fn test_has_resource_not_exists() {
        let world = World::new();

        assert!(!world.has_resource::<GameTime>());
        assert!(!world.has_resource::<PlayerScore>());
        assert!(!world.has_resource::<GameSettings>());
    }

    #[test]
    fn test_has_resource_after_removal() {
        let mut world = World::new();
        let score = PlayerScore {
            value: 500,
            high_score: 1000,
        };
        world.insert_resource(score);

        assert!(world.has_resource::<PlayerScore>());

        world.remove_resource::<PlayerScore>();
        assert!(!world.has_resource::<PlayerScore>());
    }

    #[test]
    fn test_update_resource_success() {
        let mut world = World::new();
        let initial_score = PlayerScore {
            value: 100,
            high_score: 500,
        };
        world.insert_resource(initial_score);

        let result = world.update_resource::<PlayerScore, _>(|mut score| {
            score.value += 50;
            if score.value > score.high_score {
                score.high_score = score.value;
            }
            score
        });

        assert!(result.is_ok());
        let updated_score = result.unwrap();
        assert_eq!(updated_score.value, 150);
        assert_eq!(updated_score.high_score, 500);

        // Verify the resource was actually updated in the world
        let stored_score = world.get_resource::<PlayerScore>().unwrap();
        assert_eq!(stored_score.value, 150);
        assert_eq!(stored_score.high_score, 500);
    }

    #[test]
    fn test_update_resource_new_high_score() {
        let mut world = World::new();
        let initial_score = PlayerScore {
            value: 450,
            high_score: 500,
        };
        world.insert_resource(initial_score);

        let result = world.update_resource::<PlayerScore, _>(|mut score| {
            score.value += 100; // This should exceed high_score
            if score.value > score.high_score {
                score.high_score = score.value;
            }
            score
        });

        assert!(result.is_ok());
        let updated_score = result.unwrap();
        assert_eq!(updated_score.value, 550);
        assert_eq!(updated_score.high_score, 550); // High score should be updated
    }

    #[test]
    fn test_update_resource_not_exists() {
        let mut world = World::new();

        let result = world.update_resource::<GameTime, _>(|mut time| {
            time.delta += 0.001;
            time
        });

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ComponentError::ComponentNotFound);
    }

    #[test]
    fn test_update_resource_complex_transformation() {
        let mut world = World::new();
        let time = GameTime {
            delta: 0.016,
            total: 100.0,
        };
        world.insert_resource(time);

        let result = world.update_resource::<GameTime, _>(|mut time| {
            time.total += 1.0; // Add 1 second to total time
            time.delta = if time.total > 100.0 { 0.033 } else { 0.016 }; // Change delta based on total
            time
        });

        assert!(result.is_ok());
        let updated_time = result.unwrap();
        assert_eq!(updated_time.delta, 0.033); // Should have changed to 0.033
        assert_eq!(updated_time.total, 101.0); // Should be exactly 101.0
    }

    #[test]
    fn test_multiple_resource_types() {
        let mut world = World::new();

        // Insert different resource types
        let game_time = GameTime {
            delta: 0.016,
            total: 50.0,
        };
        let player_score = PlayerScore {
            value: 1000,
            high_score: 1500,
        };
        let settings = GameSettings {
            volume: 0.7,
            difficulty: 2,
        };

        world.insert_resource(game_time.clone());
        world.insert_resource(player_score.clone());
        world.insert_resource(settings.clone());

        // All should exist independently
        assert!(world.has_resource::<GameTime>());
        assert!(world.has_resource::<PlayerScore>());
        assert!(world.has_resource::<GameSettings>());

        // Values should be correct
        assert_eq!(world.get_resource::<GameTime>().unwrap(), &game_time);
        assert_eq!(world.get_resource::<PlayerScore>().unwrap(), &player_score);
        assert_eq!(world.get_resource::<GameSettings>().unwrap(), &settings);
    }

    #[test]
    fn test_resource_independence() {
        let mut world = World::new();

        let game_time = GameTime {
            delta: 0.016,
            total: 25.0,
        };
        let player_score = PlayerScore {
            value: 500,
            high_score: 800,
        };

        world.insert_resource(game_time);
        world.insert_resource(player_score);

        // Remove one resource
        world.remove_resource::<GameTime>();

        // Other resource should remain unaffected
        assert!(!world.has_resource::<GameTime>());
        assert!(world.has_resource::<PlayerScore>());
        assert_eq!(world.get_resource::<PlayerScore>().unwrap().value, 500);
    }

    #[test]
    fn test_resource_lifecycle() {
        let mut world = World::new();
        let input = InputState {
            mouse_x: 200.0,
            mouse_y: 150.0,
            keys_pressed: vec![],
        };

        // Initial state: no resource
        assert!(!world.has_resource::<InputState>());
        assert!(world.get_resource::<InputState>().is_none());

        // After insertion
        world.insert_resource(input.clone());
        assert!(world.has_resource::<InputState>());
        assert_eq!(world.get_resource::<InputState>().unwrap(), &input);

        // After update
        world
            .update_resource::<InputState, _>(|mut state| {
                state.mouse_x += 10.0;
                state.keys_pressed.push("SPACE".to_string());
                state
            })
            .unwrap();

        let updated = world.get_resource::<InputState>().unwrap();
        assert_eq!(updated.mouse_x, 210.0);
        assert_eq!(updated.keys_pressed, vec!["SPACE".to_string()]);

        // After removal
        let removed = world.remove_resource::<InputState>().unwrap();
        assert_eq!(removed.mouse_x, 210.0);
        assert!(!world.has_resource::<InputState>());
        assert!(world.get_resource::<InputState>().is_none());
    }

    #[test]
    fn test_resource_replacement_lifecycle() {
        let mut world = World::new();

        // Insert initial resource
        let initial_settings = GameSettings {
            volume: 0.5,
            difficulty: 1,
        };
        world.insert_resource(initial_settings);
        assert_eq!(world.get_resource::<GameSettings>().unwrap().volume, 0.5);

        // Replace with new resource (insert_resource should replace)
        let new_settings = GameSettings {
            volume: 0.8,
            difficulty: 3,
        };
        world.insert_resource(new_settings);
        assert_eq!(world.get_resource::<GameSettings>().unwrap().volume, 0.8);
        assert_eq!(world.get_resource::<GameSettings>().unwrap().difficulty, 3);

        // Update the replaced resource
        world
            .update_resource::<GameSettings, _>(|mut settings| {
                settings.volume = 1.0;
                settings
            })
            .unwrap();

        assert_eq!(world.get_resource::<GameSettings>().unwrap().volume, 1.0);
        assert_eq!(world.get_resource::<GameSettings>().unwrap().difficulty, 3);
    }

    #[test]
    fn test_empty_world() {
        let world = World::new();

        // Empty world should not have any resources
        assert!(!world.has_resource::<GameTime>());
        assert!(!world.has_resource::<PlayerScore>());
        assert!(!world.has_resource::<GameSettings>());
        assert!(!world.has_resource::<InputState>());

        // Getting non-existent resources should return None
        assert!(world.get_resource::<GameTime>().is_none());
        assert!(world.get_resource::<PlayerScore>().is_none());
    }

    #[test]
    fn test_resource_immutable_access() {
        let mut world = World::new();
        let time = GameTime {
            delta: 0.016,
            total: 75.0,
        };
        world.insert_resource(time.clone());

        // Multiple immutable accesses should work
        let ref1 = world.get_resource::<GameTime>().unwrap();
        let ref2 = world.get_resource::<GameTime>().unwrap();

        assert_eq!(ref1, ref2);
        assert_eq!(ref1, &time);
    }

    #[test]
    fn test_update_resource_returns_new_value() {
        let mut world = World::new();
        let score = PlayerScore {
            value: 100,
            high_score: 200,
        };
        world.insert_resource(score);

        let updated_score = world
            .update_resource::<PlayerScore, _>(|mut s| {
                s.value *= 2;
                s
            })
            .unwrap();

        // The returned value should match the new stored value
        assert_eq!(updated_score.value, 200);
        assert_eq!(world.get_resource::<PlayerScore>().unwrap().value, 200);
    }

    #[test]
    fn test_resource_clone_requirement() {
        // This test verifies that update_resource works with Clone types
        let mut world = World::new();

        // InputState implements Clone
        let input = InputState {
            mouse_x: 0.0,
            mouse_y: 0.0,
            keys_pressed: vec!["CTRL".to_string()],
        };
        world.insert_resource(input);

        let result = world.update_resource::<InputState, _>(|mut state| {
            state.keys_pressed.push("C".to_string());
            state
        });

        assert!(result.is_ok());
        assert_eq!(
            world.get_resource::<InputState>().unwrap().keys_pressed,
            vec!["CTRL".to_string(), "C".to_string()]
        );
    }
}
