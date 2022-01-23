use futures::{stream::iter, StreamExt};
use regex::Regex;
use reqwest::{header::COOKIE, redirect::Policy, Client};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, SystemTime};
use tokio::time::sleep;

use crate::consts::USER_AGENT;
use crate::storage::Episode;

pub struct AnimePartialVariant {
    pub id: String,
    pub label: String,
    pub episodes: Vec<PartialEpisode>,
}

#[derive(Deserialize, Serialize)]
pub struct PartialEpisode {
    kind: EpKind,
    label: String,
    link: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EpKind {
    Gogo,
    VvidCC,
}

#[derive(Deserialize)]
struct _Anime {
    #[serde(rename = "title")]
    _title: Option<String>,
    #[serde(rename = "id")]
    _id: Option<String>,
    #[serde(rename = "e")]
    _e: Option<String>,
}

pub async fn parse_anime_list(testing: bool) -> Vec<(String, String, String)> {
    match testing {
        true => std::fs::read_to_string("test_local.json")
            .map(|x| serde_json::from_str::<Vec<_Anime>>(&x).unwrap())
            .unwrap(),
        false => match Client::new()
            .get("https://animixplay.to/assets/s/all.json")
            .header("user-agent", USER_AGENT)
            .header(COOKIE, "animix_ses=qie32m93lh7jhvo3d63r8pj1i06pa4pbsso9")
            .send()
            .await
            .unwrap()
            .text()
            .await
        {
            Ok(n) => serde_json::from_str(&n).expect(&format!("{}", n)),
            Err(e) => {
                panic!("Failed to download all.json: {:?}", e);
            }
        },
    }
    .into_iter()
    .filter_map(|x| {
        (x._title.is_some() && x._id.is_some() && x._e.is_some() && x._e.as_ref().unwrap() == "1")
            .then(|| (x._title.unwrap(), x._id.unwrap(), x._e.unwrap()))
    })
    .collect::<Vec<_>>()
}

pub async fn get_partial_data(
    anime_list: Vec<(String, String, String)>,
    testing: bool,
) -> Vec<(u32, AnimePartialVariant)> {
    let number_completed = AtomicUsize::new(1);
    let total_length = anime_list.len();

    iter(anime_list[420..500].iter().map(|(label, id, _)| async {
        let (animix_response, episodes) = loop {
            let animix_response = Client::new()
                .get(format!("https://animixplay.to/v1/{}", id.to_owned()))
                .header("user-agent", USER_AGENT)
                .header(COOKIE, "animix_ses=kfgub00g7eiba0l8p4br88cps3aqcir1ljqs")
                .send()
                .await
                .unwrap()
                .text()
                .await
                .unwrap();

            if testing {
                break (
                    animix_response,
                    serde_json::from_str(
                        &std::fs::read_to_string(format!("testing/{}", id.to_owned())).unwrap(),
                    )
                    .unwrap(),
                );
            } else {
                match get_episodes(&animix_response) {
                    Ok(n) => break (animix_response, n),
                    Err(_) => {
                        sleep(Duration::from_secs(2)).await;
                    }
                }
            }
        };

        //std::fs::write(
        //    format!("testing/{}", id),
        //    serde_json::to_string_pretty(&episodes).unwrap(),
        //)
        //.unwrap();

        let malid = Regex::new(r"var malid = '(.*?)';")
            .unwrap()
            .captures(&animix_response)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .parse::<u32>()
            .unwrap();

        println!(
            "[{}/{}] Done getting partial data for {}",
            number_completed.fetch_add(1, Ordering::SeqCst),
            total_length,
            malid.to_owned(),
        );

        (
            malid,
            AnimePartialVariant {
                id: id.to_owned(),
                label: label.to_owned(),
                episodes,
            },
        )
    }))
    .buffer_unordered(5)
    .collect::<Vec<_>>()
    .await
}

pub async fn scrape_episodes(episodes: Vec<PartialEpisode>) -> Vec<Episode> {
    iter(episodes.into_iter().map(|x| async move {
        match x.kind {
            EpKind::VvidCC => Some((x.label, x.link)),
            EpKind::Gogo => {
                let id = match Regex::new(r"streaming\.php\?id=(.*?)&")
                    .unwrap()
                    .captures(&x.link)
                    .map(|x| x.get(1).map(|x| x.as_str()))
                    .flatten()
                {
                    Some(n) => n,
                    None => {
                        // TODO: Error handling
                        return None;
                    }
                };

                let link = format!(
                    "https://animixplay.to/api/live{}",
                    base64::encode(
                        format!("{}LTXs3GrU8we9O{}", id, base64::encode(id.as_bytes())).as_bytes()
                    )
                );

                Some((
                    x.label,
                    String::from_utf8(
                        base64::decode(loop {
                            match Client::builder()
                                .user_agent(USER_AGENT)
                                .redirect(Policy::none())
                                .build()
                                .unwrap()
                                .get(&link)
                                .send()
                                .await
                                .unwrap()
                                .headers()
                                .get("location")
                                .map(|x| {
                                    x.to_str().ok().map(|x| x.split("#").collect::<Vec<_>>()[1])
                                })
                                .flatten()
                            {
                                Some(n) => break n.to_owned(),
                                None => sleep(Duration::from_secs(3)).await,
                            }
                        })
                        .unwrap(),
                    )
                    .unwrap(),
                ))
            }
        }
    }))
    .buffer_unordered(5)
    .filter_map(|x| async {
        x.map(|(label, link)| Episode {
            label,
            link,
            added_in: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    })
    .collect::<Vec<Episode>>()
    .await
}

fn get_episodes(response: &str) -> Result<Vec<PartialEpisode>, String> {
    let html = Html::parse_document(response);
    let selector = Selector::parse("#epslistplace").unwrap();
    let results = html.select(&selector).collect::<Vec<_>>();

    if results.len() < 1 {
        return Err(format!("#epslistplace not found"));
    }

    let value = match serde_json::from_str::<Value>(&results[0].inner_html()) {
        Ok(n) => n,
        Err(e) => return Err(format!("couldn't parse json: {}", e)),
    };

    let map = if results.len() > 0 {
        match value.as_object() {
            Some(n) => n,
            None => return Err(format!("not a JSON object")),
        }
    } else {
        return Err(format!("#epslistplace not found"));
    };

    let mut episodes = Vec::new();

    episodes.extend(
        map.iter()
            .filter(|(k, _)| !k.contains("eptotal"))
            .filter_map(|(k, v)| v.as_str().map(|v| (k, v))),
    );

    episodes.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(episodes
        .into_iter()
        .filter_map(|(a, b)| {
            if b.contains("gogoplay.io") {
                Some(PartialEpisode {
                    kind: EpKind::Gogo,
                    label: a.to_owned(),
                    link: format!("https:{}", b),
                })
            } else if b.contains("v.vvid.cc") {
                Some(PartialEpisode {
                    kind: EpKind::VvidCC,
                    label: a.to_owned(),
                    link: b.to_owned(),
                })
            } else {
                println!("Unknown ep kind ({}): {}", a, b);
                None
            }
        })
        .collect())
}
