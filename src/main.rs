use chrono::NaiveDateTime;
use dialoguer::{Input, Select};
use dotenv::dotenv;
use regex::Regex;
use serde::Deserialize;
use std::env;
use std::fs;
use walkdir::WalkDir;

#[derive(Debug, Deserialize)]
struct FrontMatter {
    date: Option<String>,
    XP: Option<u32>,
    // Добавьте другие поля, если необходимо
}

fn parse_yaml_front_matter(content: &str) -> Result<(FrontMatter, String), Box<dyn std::error::Error>> {
    let re = Regex::new(r"(?s)^---\n(.*?)\n---\n(.*)")?;

    if let Some(captures) = re.captures(content) {
        let yaml_str = &captures[1];
        let rest_of_content = &captures[2];

        let front_matter: FrontMatter = serde_yaml::from_str(yaml_str)?;

        Ok((front_matter, rest_of_content.to_string()))
    } else {
        Ok((FrontMatter { date: None, XP: None }, content.to_string()))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let vault_path = env::var("OBSIDIAN_VAULT_PATH").expect("OBSIDIAN_VAULT_PATH не установлен в .env");

    let experience_path = format!("{}/Experience", vault_path);

    // Суммируем XP и вычисляем уровень
    let mut total_xp = 0;

    for entry in WalkDir::new(&experience_path) {
        let entry = entry?;
        if entry.file_type().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(entry.path())?;
            let (front_matter, _) = parse_yaml_front_matter(&content)?;

            if let Some(xp) = front_matter.XP {
                total_xp += xp;
            }
        }
    }

    let level = total_xp / 10_000;
    println!("Ваш текущий уровень: {}", level);

    // Меню
    let options = vec!["Напечатать все опыты начиная с даты", "Выход"];
    let selection = Select::new()
        .with_prompt("Выберите опцию")
        .items(&options)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            // Запрашиваем дату у пользователя
            let date_input: String = Input::new()
                .with_prompt("Введите дату в формате ГГГГ-ММ-ДД")
                .interact_text()?;

            let cutoff_date = NaiveDateTime::parse_from_str(&format!("{}T00:00:00", date_input), "%Y-%m-%dT%H:%M:%S")?;

            let mut experiences = Vec::new();

            for entry in WalkDir::new(&experience_path) {
                let entry = entry?;
                if entry.file_type().is_file() && entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
                    let content = fs::read_to_string(entry.path())?;
                    let (front_matter, body) = parse_yaml_front_matter(&content)?;

                    if let Some(date_str) = front_matter.date {
                        if let Ok(file_date) = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S") {
                            if file_date >= cutoff_date {
                                experiences.push((file_date, body));
                            } else {
                                continue;
                            }
                        } else if let Ok(file_date) = NaiveDateTime::parse_from_str(&format!("{}T00:00:00", date_str), "%Y-%m-%dT%H:%M:%S") {
                            if file_date >= cutoff_date {
                                experiences.push((file_date, body));
                            } else {
                                continue;
                            }
                        } else {
                            println!("Неверный формат даты в файле {:?}", entry.path());
                        }
                    }
                }
            }

            // Сортируем опыты по дате
            experiences.sort_by_key(|(date, _)| *date);

            // Записываем опыты в файл
            let output_file = "filtered_experiences.md";
            let mut output = fs::File::create(output_file)?;

            for (_, body) in experiences {
                use std::io::Write;
                writeln!(output, "{}", body)?;
                writeln!(output, "\n---\n")?;
            }

            println!("Опыты записаны в файл {}", output_file);
        }
        _ => {
            println!("Выход из программы.");
        }
    }

    Ok(())
}
