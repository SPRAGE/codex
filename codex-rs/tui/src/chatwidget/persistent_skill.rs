use std::collections::HashSet;
use std::path::Path;

use codex_app_server_protocol::UserInput;
use codex_core_skills::model::SkillMetadata;
use codex_protocol::config_types::CollaborationMode;
use codex_utils_absolute_path::AbsolutePathBuf;

use super::ChatWidget;
use crate::skills_helpers::skill_display_name;

const PERSISTENT_SKILL_USAGE: &str = "Usage: /persistent-skill <skill-name|path|status|clear>";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PersistentSkillSelection {
    pub(super) name: String,
    pub(super) path: AbsolutePathBuf,
}

impl PersistentSkillSelection {
    fn from_skill(skill: &SkillMetadata) -> Self {
        Self {
            name: skill.name.clone(),
            path: skill.path_to_skills_md.clone(),
        }
    }

    pub(super) fn input_item(&self) -> UserInput {
        UserInput::Skill {
            name: self.name.clone(),
            path: self.path.to_path_buf(),
        }
    }

    fn developer_instructions(&self) -> String {
        let name = &self.name;
        format!(
            "Persistent skill mode is active for `{name}`. Treat every user turn as an invocation of this skill. The full SKILL.md content is loaded on the next model turn after activation; this compact guardrail keeps that skill as the sole conversational frame on later turns. If the skill defines an organization, roles, or personas, respond only as those user-facing members. Do not answer in the default Codex voice except for mandatory safety and tool constraints from higher-priority instructions."
        )
    }
}

impl ChatWidget {
    pub(super) fn handle_persistent_skill_command(&mut self, args: &str) {
        let trimmed = args.trim();
        match trimmed.to_ascii_lowercase().as_str() {
            "" | "status" => {
                self.add_persistent_skill_status();
            }
            "clear" | "off" | "none" => {
                if let Some(selection) = self.persistent_skill.take() {
                    self.persistent_skill_pending_injection = false;
                    self.add_info_message(
                        format!("Persistent skill cleared: {}", selection.name),
                        Some(PERSISTENT_SKILL_USAGE.to_string()),
                    );
                } else {
                    self.add_info_message(
                        "No persistent skill is active.".to_string(),
                        Some(PERSISTENT_SKILL_USAGE.to_string()),
                    );
                }
            }
            _ => match self.resolve_persistent_skill(trimmed) {
                Ok(selection) => {
                    let name = selection.name.clone();
                    self.persistent_skill = Some(selection);
                    self.persistent_skill_pending_injection = true;
                    self.add_info_message(
                        format!("Persistent skill active: {name}"),
                        Some("The next turn will load this skill; later turns keep a compact persistent guardrail.".to_string()),
                    );
                }
                Err(message) => self.add_error_message(message),
            },
        }
    }

    pub(super) fn ensure_persistent_skill_item(
        &self,
        items: &mut Vec<UserInput>,
        selected_skill_paths: &mut HashSet<AbsolutePathBuf>,
    ) -> bool {
        let Some(selection) = self.persistent_skill.as_ref() else {
            return false;
        };
        if !self.persistent_skill_pending_injection {
            return false;
        }
        if selected_skill_paths.insert(selection.path.clone()) {
            items.push(selection.input_item());
        }
        true
    }

    pub(super) fn mark_persistent_skill_injected(&mut self) {
        self.persistent_skill_pending_injection = false;
    }

    pub(super) fn collaboration_mode_with_persistent_skill(
        &self,
        effective_mode: &CollaborationMode,
    ) -> Option<CollaborationMode> {
        let base_mode =
            if self.collaboration_modes_enabled() && self.active_collaboration_mask.is_some() {
                Some(effective_mode.clone())
            } else {
                None
            };
        let Some(selection) = self.persistent_skill.as_ref() else {
            return base_mode;
        };
        let mode = base_mode.unwrap_or_else(|| effective_mode.clone());
        let developer_instructions = merge_developer_instructions(
            mode.settings.developer_instructions.as_deref(),
            &selection.developer_instructions(),
        );
        Some(mode.with_updates(
            /*model*/ None,
            /*effort*/ None,
            Some(Some(developer_instructions)),
        ))
    }

    fn add_persistent_skill_status(&mut self) {
        if let Some(selection) = self.persistent_skill.as_ref() {
            self.add_info_message(
                format!("Persistent skill active: {}", selection.name),
                Some(format!("Path: {}", selection.path.display())),
            );
        } else {
            self.add_info_message(
                "No persistent skill is active.".to_string(),
                Some(PERSISTENT_SKILL_USAGE.to_string()),
            );
        }
    }

    fn resolve_persistent_skill(&self, query: &str) -> Result<PersistentSkillSelection, String> {
        let query = query.trim().trim_start_matches('$');
        if query.is_empty() {
            return Err(PERSISTENT_SKILL_USAGE.to_string());
        }
        let Some(skills) = self.bottom_pane.skills() else {
            return Err(
                "No skills are available yet. Try again after startup finishes.".to_string(),
            );
        };
        if skills.is_empty() {
            return Err("No enabled skills are available. Use /skills to enable one.".to_string());
        }

        if let Some(path) = self.query_as_skill_path(query)
            && let Some(skill) = skills.iter().find(|skill| skill.path_to_skills_md == path)
        {
            return Ok(PersistentSkillSelection::from_skill(skill));
        }

        let matches = skills
            .iter()
            .filter(|skill| skill_matches_query(skill, query))
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [skill] => Ok(PersistentSkillSelection::from_skill(skill)),
            [] => Err(format!(
                "No enabled skill matches `{query}`. Use /skills to view or enable skills."
            )),
            _ => Err(format!(
                "Multiple enabled skills match `{query}`. Use /persistent-skill <full path>."
            )),
        }
    }

    fn query_as_skill_path(&self, query: &str) -> Option<AbsolutePathBuf> {
        let query = query.strip_prefix("skill://").unwrap_or(query);
        if !query.contains('/') && !query.contains('\\') {
            return None;
        }
        Some(self.config.cwd.join(Path::new(query)))
    }
}

fn skill_matches_query(skill: &SkillMetadata, query: &str) -> bool {
    skill.name.eq_ignore_ascii_case(query) || skill_display_name(skill).eq_ignore_ascii_case(query)
}

fn merge_developer_instructions(existing: Option<&str>, addition: &str) -> String {
    existing
        .filter(|instructions| !instructions.trim().is_empty())
        .map_or_else(
            || addition.to_string(),
            |instructions| format!("{instructions}\n\n{addition}"),
        )
}
