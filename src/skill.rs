use std::path::Path;

use anyhow::{Result, anyhow};
use tokio::fs;

// struct Skill<'a> {
//     name: &'a str,
//     description: &'a str,
//     body: &'a str,
// }

pub struct Skill {
    pub name: String,
    pub description: String,
    pub body: String,
}

impl Skill {
    fn new(source: &str) -> Result<Self> {
        let mut delim_count = 0;
        let mut in_heading = false;
        let mut total_bytes = 0;

        let mut name = None;
        let mut description = None;
        let mut body = None;

        for line in source.lines() {
            if line == "---" {
                delim_count += 1;
                in_heading = delim_count < 2;
            } else if in_heading && let Some((key, value)) = line.split_once(':') {
                if key == "name" {
                    name.replace(value.trim());
                } else if key == "description" {
                    description.replace(value.trim());
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
        })
    }
}

pub async fn skills() -> Result<Vec<Skill>> {
    async fn walk(dir: &Path, visit: &mut impl AsyncFnMut(&Path) -> Result<()>) -> Result<()> {
        let mut dirs = fs::read_dir(dir).await?;
        while let Some(entry) = dirs.next_entry().await? {
            let path = entry.path();

            // file_type() doesn't follow symlinks, so this avoids symlink loops
            let file_type = entry.file_type().await?;
            if file_type.is_dir() {
                Box::pin(walk(&path, visit)).await?;
            } else if file_type.is_file() {
                visit(&path).await?;
            }
        }
        Ok(())
    }

    let mut skills: Vec<Skill> = Vec::new();

    walk(Path::new(".claude/skills"), &mut async |path| {
        let source = fs::read_to_string(path).await?;
        skills.push(Skill::new(&source)?);

        Ok(())
    })
    .await?;

    Ok(skills)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generic_skill_parse_success() -> Result<()> {
        let source = "---\n\
                            name: banana\n\
                            description: Instructs how to eat a banana.\n\
                            ---\n\
                            Remove the banana peel.\n\
                            Then bite the banana and chew the piece you've bitten.\n\
                            Gulp it.\n\
                            Repeat until the entire banana it eaten.";

        let skill = Skill::new(source)?;

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
}
