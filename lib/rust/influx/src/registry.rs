use std::collections::HashMap;

use flatten_json_object::{ArrayFormatting, Flattener};

use crate::{LogPolicy, PolicyRouter};

pub(crate) struct ValueRegistry {
    registry: HashMap<String, LogPolicy>,
    policy_router: PolicyRouter,
}

impl ValueRegistry {
    pub(crate) fn new(policy_router: PolicyRouter) -> Self {
        Self {
            registry: HashMap::new(),
            policy_router,
        }
    }

    pub(crate) fn update<T: serde::Serialize>(&mut self, values: T) -> Vec<(String, serde_json::Value)> {
        let flat = Flattener::new()
            .set_preserve_empty_objects(true)
            .set_preserve_empty_objects(true)
            .set_array_formatting(ArrayFormatting::Surrounded {
                start: "[".to_string(),
                end: "]".to_string(),
            })
            .flatten(&serde_json::to_value(values).expect("serde doesn't fail"))
            .expect("no key collisions");

        let new_measurements = flat.as_object().expect("output of .flatten is an object");

        // Determine which measurements should be sent to the database
        let mut points_to_write = Vec::new();

        for (key, val) in new_measurements {
            let policy = self
                .registry
                .entry(key.clone())
                .or_insert_with(|| self.policy_router.policy_for(key).clone());

            if policy.should_log(val) {
                points_to_write.push((key.clone(), val.clone()));
            }
        }

        points_to_write
    }
}
