use globset::{Glob, GlobMatcher};

use super::LogPolicy;

pub struct PolicyRouter {
    rules: Vec<(GlobMatcher, LogPolicy)>,
    default: LogPolicy,
}

impl PolicyRouter {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            default: LogPolicy::default(),
        }
    }

    /// Set the policy applied to keys which match no rules
    pub fn with_default(mut self, default_policy: LogPolicy) -> Self {
        self.default = default_policy;
        self
    }

    /// Set a specific `policy` for keys matching glob `pattern`
    ///
    /// The first matching policy will be selected when a key matches multiple patterns.
    pub fn rule(mut self, pattern: &str, policy: LogPolicy) -> Self {
        // glob considers `[` and `]` special characters, but json arrays get flattened to
        // `arr[0], arr[1], ...`. Escape brackets for ease of use.
        let pattern = pattern.replace('[', r"\[").replace(']', r"\]").clone();
        let matcher = Glob::new(&pattern).expect("a valid glob pattern").compile_matcher();
        self.rules.push((matcher, policy));
        self
    }

    pub(crate) fn policy_for(&self, key: &str) -> &LogPolicy {
        self.rules
            .iter()
            .find(|(pattern, _)| pattern.is_match(key))
            .map_or(&self.default, |(_, policy)| policy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn rules() {
        let router = PolicyRouter::new()
            .with_default(LogPolicy::EveryMeasurement)
            .rule("*state*", LogPolicy::on_change(Duration::from_millis(1000)))
            .rule(
                "sensors.important",
                LogPolicy::after_interval(Duration::from_millis(10)),
            )
            .rule("sensors.*", LogPolicy::on_change(Duration::from_millis(1000)))
            .rule("child[0].*", LogPolicy::EveryMeasurement)
            .rule("child*.*", LogPolicy::after_interval(Duration::from_millis(2000)));

        assert!(matches!(*router.policy_for("not_a_match"), LogPolicy::EveryMeasurement));
        assert!(matches!(
            *router.policy_for("sensors.important"),
            LogPolicy::AfterInterval(_)
        ));
        assert!(matches!(
            *router.policy_for("sensors.not_important"),
            LogPolicy::OnChange(_)
        ));
        assert!(
            matches!(*router.policy_for("child[0].state"), LogPolicy::OnChange(_)),
            "`state` has a separate rule"
        );
        assert!(matches!(
            *router.policy_for("child[0].data"),
            LogPolicy::EveryMeasurement
        ));
        assert!(
            matches!(*router.policy_for("child[1].state"), LogPolicy::OnChange(_)),
            "`state` has a separate rule"
        );
        assert!(matches!(
            *router.policy_for("child[1].data"),
            LogPolicy::AfterInterval(_)
        ));
    }
}
