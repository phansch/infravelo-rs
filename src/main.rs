use reqwest::{Result, StatusCode};
use std::fs;
use chrono::prelude::*;
use serde_derive::{Serialize, Deserialize};

#[derive(Deserialize)]
struct ApiResponse {
    #[serde(rename(deserialize = "results"))]
    projects: Vec<Project>,
    previous: Option<String>,
    next: Option<String>,
}

// Use jd to look at differences for a few days: https://github.com/josephburnett/jd#command-line-usage
#[derive(Serialize, Deserialize, Debug)]
struct Project {
    id: String,
    title: String,
    subtitle: String,
    description: String,
    status: Option<String>,
    holder: String,
    costs: String,
    link: String,
    #[serde(rename(deserialize = "apiLink"))]
    api_link: String,
    #[serde(rename(deserialize = "yearOfImplementation"))]
    year_of_implementation: Option<i32>,
    #[serde(rename(deserialize = "companyConstruction"))]
    company_construction: String,
    #[serde(rename(deserialize = "companyPlanning"))]
    company_planning: String,
    owner: String,
    coordinator: String,
    date_start: Option<String>,
    date_end: Option<String>,
    districts: Vec<District>,
    image: Option<Image>,
    kml: String
}

#[derive(Serialize, Deserialize, Debug)]
struct Image {
    uri: String,
    extension: String
}

#[derive(Serialize, Deserialize, Debug)]
struct District {
    name: String
}


struct ProjectsApi {
    projects: <Vec<Project> as IntoIterator>::IntoIter,
    client: reqwest::blocking::Client,
    next: Option<String>,
    previous: Option<String>,
}

impl ProjectsApi {
    fn of() -> Result<Self> {
        Ok(ProjectsApi {
               projects: vec![].into_iter(),
               client: reqwest::blocking::Client::new(),
               next: Some(String::from("https://www.infravelo.de/api/v1/projects/")),
               previous: None,
           })
    }

    fn try_next(&mut self) -> Result<Option<Project>> {
        if let Some(project) = self.projects.next() {
            return Ok(Some(project));
        }

        let url = self.next.clone();
        let response = self.client.get(&url.expect("foo")).send()?;
        if response.status() == StatusCode::NOT_FOUND { return Ok(None) }

        let parsed_response = response.json::<ApiResponse>()?;


        self.projects = parsed_response.projects.into_iter();

        if parsed_response.next.is_none() { return Ok(None)}

        self.next = parsed_response.next;
        self.previous = parsed_response.previous;

        Ok(self.projects.next())
    }
}

impl Iterator for ProjectsApi {
    type Item = Result<Project>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(Some(dep)) => Some(Ok(dep)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

fn filepath() -> String {
    let today = Local::now().date_naive();
    format!("/tmp/infravelo-{}.json", today.format("%d-%m-%Y"))

}
fn main() -> Result<()> {
    let mut projects: Vec<Project> = vec![];
    for project in ProjectsApi::of()? {

        match project {
            Ok(project) => {
                println!("Project written");
                projects.push(project);
            }
            Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
        }

    }

    println!("{} projects", projects.len());

    let output = serde_json::to_string(&projects).unwrap();
    fs::write(filepath(), output).expect("Unable to write file");
    println!("Written to {}", filepath());

    Ok(())
}
