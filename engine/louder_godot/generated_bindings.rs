use louder_rust::godot_api;
use godot::prelude::*;

#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct Motion {
    pub(crate) inner: engine::Motion,
}

godot_api! {
    impl Motion {
        #[func]
        pub fn get_source_position(&self) -> engine::LouderPosition {
            self.inner.source_position.clone()
        }

        #[func]
        pub fn get_target_position(&self) -> engine::LouderPosition {
            self.inner.target_position.clone()
        }

        #[func]
        pub fn get_choice_index(&self) -> i32 {
            self.inner.choice_index.clone()
        }

        #[func]
        pub fn get_speaker(&self) -> engine::LouderName {
            self.inner.speaker.clone()
        }

        #[func]
        pub fn get_background(&self) -> bool {
            self.inner.background.clone()
        }

        #[func]
        pub fn get_rule(&self) -> i32 {
            self.inner.rule.clone() as i32
        }

    }
}

#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct AdditionalDataChange {
    pub(crate) inner: engine::AdditionalDataChange,
}

godot_api! {
    impl AdditionalDataChange {
        #[func]
        pub fn get_key(&self) -> engine::LouderName {
            self.inner.key.clone()
        }

        #[func]
        pub fn get_rule(&self) -> i32 {
            self.inner.rule.clone() as i32
        }

    }
}

#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct LoadFailure {
    pub(crate) inner: engine::LoadFailure,
}

godot_api! {
    impl LoadFailure {
        #[func]
        pub fn get_position(&self) -> engine::LouderPosition {
            self.inner.position.clone()
        }

        #[func]
        pub fn get_error(&self) -> i32 {
            self.inner.error.clone()
        }

    }
}

#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct ChoiceAppearance {
    pub(crate) inner: engine::ChoiceAppearance,
}

godot_api! {
    impl ChoiceAppearance {
        #[func]
        pub fn get_position(&self) -> engine::LouderPosition {
            self.inner.position.clone()
        }

        #[func]
        pub fn get_choice_index(&self) -> i32 {
            self.inner.choice_index.clone()
        }

        #[func]
        pub fn get_rule(&self) -> i32 {
            self.inner.rule.clone() as i32
        }

    }
}

#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct ConditionOutcome {
    pub(crate) inner: engine::ConditionOutcome,
}

godot_api! {
    impl ConditionOutcome {
        #[func]
        pub fn get_condition(&self) -> engine::LouderName {
            self.inner.condition.clone()
        }

    }
}

#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct Interrupt {
    pub(crate) inner: engine::Interrupt,
}

godot_api! {
    impl Interrupt {
        #[func]
        pub fn get_actor(&self) -> engine::LouderActor {
            self.inner.actor.clone()
        }

        #[func]
        pub fn get_expression(&self) -> engine::LouderName {
            self.inner.expression.clone()
        }

    }
}

