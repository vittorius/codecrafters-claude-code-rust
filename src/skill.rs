use std::{collections::HashMap, path::Path};

use anyhow::{Result, anyhow};
use tokio::fs;

const BASE_PATH: &str = ".claude/skills";

#[derive(Debug)]
pub struct Skill {
    pub name: String,
    pub description: String,
    body: String,
    // this field prevents the model from attempting to read the skill dir structure
    // if it doesn't contain any scripts (meant to be invoked by a user)
    has_scripts: bool,
    in_subagent: bool,
}

impl Skill {
    // passing source by value because we make the Skill own the fragments of the source String instead
    pub fn new(source: &str, has_scripts: bool) -> Result<Self> {
        let mut delim_count = 0;
        let mut in_heading = false;
        let mut total_bytes = 0;

        let mut name = None;
        let mut description = None;
        let mut in_subagent = false;
        let mut body = None;

        for line in source.lines() {
            if line == "---" {
                delim_count += 1;
                in_heading = delim_count < 2;
            } else if in_heading && let Some((key, value)) = line.split_once(':') {
                match key {
                    "name" => {
                        name.replace(value.trim());
                    }
                    "description" => {
                        description.replace(value.trim());
                    }
                    "context" if value.trim() == "fork" => {
                        in_subagent = true;
                    }
                    _ => {}
                }
            } else {
                body.replace(&source[total_bytes..]);
                break;
            }

            total_bytes += line.len();
            total_bytes += if cfg!(target_os = "windows") { 2 } else { 1 }; // add bytes for newlines omitted by .lines()
        }

        Ok(Skill {
            name: name
                .map(str::to_owned)
                .ok_or_else(|| anyhow!("Missing 'name' attribute"))?,
            description: description
                .map(str::to_owned)
                .ok_or_else(|| anyhow!("Missing 'description' attribute"))?,
            body: body
                .map(str::to_owned)
                .ok_or_else(|| anyhow!("Missing body"))?,
            has_scripts,
            in_subagent,
        })
    }

    pub fn body(&self, args_str: &str) -> String {
        let mut result = self.body.clone();
        for (i, arg) in args_str.split(' ').enumerate() {
            result = result
                .replace(&format!("$ARGUMENTS[{i}]"), arg)
                .replace(&format!("${i}"), arg);
        }
        result = result.replace("$ARGUMENTS", args_str);

        if self.has_scripts {
            format!(
                "Paths in the instructions below are relative to the {}/{} folder.\n{}",
                BASE_PATH, self.name, result
            )
        } else {
            result
        }
    }

    pub fn in_subagent(&self) -> bool {
        self.in_subagent
    }
}

pub async fn build_skills() -> Result<HashMap<String, Skill>> {
    async fn is_dir_not_empty(path: &Path) -> Result<bool> {
        let mut read_dir = fs::read_dir(path).await?;
        Ok(read_dir.next_entry().await?.is_some())
    }

    let mut skills: HashMap<String, Skill> = HashMap::new();

    let all_skills_path = Path::new(BASE_PATH);
    if all_skills_path.exists() {
        let mut dirs = fs::read_dir(all_skills_path).await?;
        while let Some(skill_dir_entry) = dirs.next_entry().await? {
            let skill_dir = skill_dir_entry.path();

            // file_type() doesn't follow symlinks, so this avoids symlink loops
            let file_type = skill_dir_entry.file_type().await?;
            if file_type.is_dir() {
                let skill_file = skill_dir.join("SKILL.md");
                if skill_file.exists() {
                    let source = fs::read_to_string(&skill_file).await?;
                    let scripts_path = skill_dir.join("scripts");

                    let skill = Skill::new(
                        &source,
                        scripts_path.exists() && is_dir_not_empty(&scripts_path).await?,
                    )?;
                    skills.insert(skill.name.clone(), skill);
                }
            }
        }
    }

    Ok(skills)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_skill_parse_success() -> Result<()> {
        #[rustfmt::skip]
        let source =
            "---\n\
            name: banana\n\
            description: Instructs how to eat a banana.\n\
            ---\n\
            Remove the banana peel.\n\
            Then bite the banana and chew the piece you've bitten.\n\
            Gulp it.\n\
            Repeat until the entire banana it eaten.";

        let skill = Skill::new(source, false)?;

        assert_eq!(skill.name, "banana");
        assert_eq!(skill.description, "Instructs how to eat a banana.");
        assert_eq!(
            skill.body,
            "Remove the banana peel.\n\
            Then bite the banana and chew the piece you've bitten.\n\
            Gulp it.\n\
            Repeat until the entire banana it eaten."
        );

        Ok(())
    }

    #[test]
    fn test_skill_arguments_substitution() -> Result<()> {
        #[rustfmt::skip]
        let source =
            "---\n\
            name: cake\n\
            description: It's a generic recipe for the cake.\n\
            ---\n\
            Take 1 spoon of $ARGUMENTS[0], 2 glasses of $ARGUMENTS[1], and a handful of $ARGUMENTS[2].\n\
            Put all of that ($ARGUMENTS) in a bowl and mix with a mixer.\n\
            Then, fill the 0.5 liter form with the mixed dough.\n\
            Put the form into the stove and set the temperature to 250 °C.\n\
            Bake for 40 minutes.\n";

        let skill = Skill::new(source, false)?;
        assert_eq!(
            skill.body("flour sugar raisins"),
            "Take 1 spoon of flour, 2 glasses of sugar, and a handful of raisins.\n\
            Put all of that (flour sugar raisins) in a bowl and mix with a mixer.\n\
            Then, fill the 0.5 liter form with the mixed dough.\n\
            Put the form into the stove and set the temperature to 250 °C.\n\
            Bake for 40 minutes.\n"
        );

        Ok(())
    }
}
