mod authorization;
mod module;

use authorization::Authorization;
use crate::Result;
use module::Module;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use reqwest::{Method};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Name {
    user_name_original: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Term {
    term_detail: TermDetail,
}

#[derive(Deserialize)]
struct TermDetail {
    term: String,
    description: String,
}

#[derive(Deserialize)]
#[derive(Debug)]
struct ApiData {
    data: Data,
}

#[derive(Debug)]
#[derive(Deserialize)]
#[serde(untagged)]
enum Data {
    Empty(Vec<String>),
    Modules(Vec<Module>),
    Announcements(Vec<Announcement>),
}

#[derive(Debug)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Announcement {
    pub title: String,
    pub description: String
}

pub struct Api {
    authorization: Authorization,
}

impl Api {
    pub fn with_login(username: &str, password: &str) -> Result<Api> {
        let mut auth = Authorization::new();
        auth.login(username, password)?;
        Ok(Api { authorization: auth })
    }

    fn api_as_json<T: DeserializeOwned>(&self, path: &str, method: Method, form: Option<&HashMap<&str, &str>>) -> Result<T> {
        Ok(self.authorization.api(path, method, form)?.json().map_err(|_|"Unable to deserialize JSON")?)
    }

    pub fn name(&self) -> Result<String> {
        let name: Name = self.api_as_json("/user/Profile", Method::GET, None)?;
        Ok(name.user_name_original)
    }

    fn current_term(&self) -> Result<String> {
        let term: Term = self.api_as_json("/setting/AcademicWeek/current?populate=termDetail", Method::GET, None)?;
        Ok(term.term_detail.term)
    }

    pub fn modules(&self, current_term_only: bool) -> Result<Vec<Module>> {
        let api_data: ApiData = self.api_as_json("/module", Method::GET, None)?;
        if let Data::Modules(modules) = api_data.data {
            if current_term_only {
                let current_term = self.current_term()?;
                Ok(modules.into_iter().filter(|m|m.term == current_term).collect())
            } else {
                Ok(modules)
            }
        } else if let Data::Empty(_) = api_data.data {
            Ok(Vec::new())
        } else {
            Err("Invalid API response from server: type mismatch")
        }
    }

    pub fn get_announcements(&self, module: &Module, archived: bool) -> Result<Vec<Announcement>> {
        let path = format!("/announcement/{}/{}?sortby=displayFrom%20ASC", if archived { "Archived" } else { "NonArchived" }, module.id);
        let api_data: ApiData = self.api_as_json(&path, Method::GET, None)?;
        if let Data::Announcements(announcements) = api_data.data {
            Ok(announcements)
        } else if let Data::Empty(_) = api_data.data {
            Ok(Vec::new())
        } else {
            Err("Invalid API response from server: type mismatch")
        }
    }
}