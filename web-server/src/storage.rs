use chashmap::CHashMap;
use futures::stream::{iter, StreamExt};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{create_dir_all, read_dir, read_to_string, write};
use std::mem::replace;
use tokio::sync::RwLock;

use crate::consts::MAL_FIELDS;
use crate::scrape::animixplay::{scrape_episodes, AnimePartialVariant};

pub struct Storage {
    pub cache: CHashMap<u32, RwLock<Option<Anime>>>,
    lists: Lists,
}

pub struct Lists {
    top_anime: Vec<u32>,
    top_airing: Vec<u32>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Anime {
    mal_details: Value,
    last_updated: u64,
    variants: Vec<AnimeVariant>,
}

#[derive(Clone, Deserialize, Serialize)]
struct AnimeVariant {
    id: String,
    label: String,
    episodes: Vec<Episode>,
    last_updated: u64,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Episode {
    pub label: String,
    pub link: String,
    pub added_in: u64,
}

impl Storage {
    pub fn new() -> Storage {
        create_dir_all("data").unwrap();

        let cache = read_dir("data")
            .unwrap()
            .filter_map(|x| {
                x.map(|x| {
                    x.file_name().to_str().map(|x| {
                        read_to_string(format!("data/{}", x))
                            .map(|json| {
                                x.parse::<u32>()
                                    .map(|malid| {
                                        serde_json::from_str::<Anime>(&json)
                                            .map(|anime| (malid, RwLock::new(Some(anime))))
                                            .map_err(|e| {
                                                println!("Error 4: {:?}", e);
                                            })
                                            .ok()
                                    })
                                    .map_err(|e| {
                                        println!("Error 3: {:?}", e);
                                    })
                                    .ok()
                            })
                            .map_err(|e| {
                                println!("Error 2: {:?}", e);
                            })
                            .ok()
                    })
                })
                .map_err(|e| {
                    println!("Error 1: {:?}", e);
                })
                .ok()
                .flatten()
                .flatten()
                .flatten()
                .flatten()
            })
            .collect::<CHashMap<_, _>>();

        let lists = Lists::load(&cache);

        Storage { cache, lists }
    }

    pub async fn get_all(&self) -> Vec<Anime> {
        let top_anime = self.get_top_anime();
        let top_airing = self.get_top_airing();

        let mut already_seen = Vec::new();
        let mut keys = [top_anime, top_airing].concat();

        keys.retain(|x| match already_seen.contains(x) {
            true => false,
            false => {
                already_seen.push(*x);
                true
            }
        });

        iter(
            keys.into_iter()
                .map(|x| async move { self.cache.get(&x).unwrap().read().await.to_owned() }),
        )
        .buffer_unordered(10)
        .filter_map(|x| async { x })
        .collect::<Vec<_>>()
        .await
    }

    pub fn get_top_anime(&self) -> &[u32] {
        self.lists.top_anime.as_slice()
    }

    pub fn get_top_airing(&self) -> &[u32] {
        self.lists.top_airing.as_slice()
    }

    pub async fn update_variant(&self, malid: u32, partial_variant: AnimePartialVariant) {
        let rwlock;

        let mut lock = match self.cache.get(&malid) {
            Some(n) => {
                rwlock = n;
                let lock = rwlock.write().await;

                match lock
                    .as_ref()
                    .unwrap()
                    .variants
                    .iter()
                    .find(|x| x.id == partial_variant.id)
                {
                    Some(n) => {
                        if n.episodes.len() == partial_variant.episodes.len() {
                            println!("Skipping malid {}", malid);
                            return;
                        }

                        println!("New episodes detected for malid {}", malid);
                    }
                    None => {
                        println!("New variant of anime for malid {}", malid);
                    }
                };

                lock
            }
            _ => {
                println!("Completely new anime (malid {})", malid);

                // To prevent multiple variants being downloaded at the same time
                // so that duplicate variants won't be possible
                self.cache.insert(malid, RwLock::new(None));
                rwlock = self.cache.get(&malid).unwrap();

                let mut lock = rwlock.write().await;

                // Get MAL details
                let mal_details = Client::new()
                    .get(format!(
                        "https://api.myanimelist.net/v2/anime/{}{}",
                        malid, MAL_FIELDS
                    ))
                    .header("X-MAL-CLIENT-ID", "800345fdcff9f9d7dbe27402f8245e4d")
                    .send()
                    .await
                    .unwrap()
                    .json::<Value>()
                    .await
                    .unwrap();

                lock.replace(Anime {
                    mal_details,
                    variants: vec![],
                    last_updated: 0,
                });

                lock
            }
        };

        println!("Refreshing episodes for malid {}", malid);

        // Refresh episodes
        let anime = lock.as_mut().unwrap();
        let episodes = scrape_episodes(partial_variant.episodes).await;

        let mut timestamps = episodes.iter().map(|x| x.added_in).collect::<Vec<_>>();
        timestamps.sort_unstable();

        let old_value = match anime
            .variants
            .iter_mut()
            .position(|x| x.id == partial_variant.id)
        {
            Some(n) => Some((n, replace(&mut anime.variants[n].episodes, episodes))),
            None => {
                anime.variants.push(AnimeVariant {
                    id: partial_variant.id,
                    label: partial_variant.label,
                    episodes,
                    last_updated: *timestamps.last().unwrap_or(&0),
                });

                None
            }
        };

        match write(
            format!("data/{}", malid),
            serde_json::to_string(anime).unwrap(),
        ) {
            Ok(_) => match old_value {
                Some((n, _)) => {
                    anime.variants[n].last_updated = *timestamps.last().unwrap_or(&0);
                }
                None => {}
            },
            Err(e) => {
                // TODO: Handle error
                match old_value {
                    Some((n, episodes)) => anime.variants[n].episodes = episodes,
                    None => {
                        anime.variants.pop();
                    }
                }

                if anime.variants.len() == 0 {
                    *lock = None;
                }
            }
        };
    }
}

impl Lists {
    pub fn load(cache: &CHashMap<u32, RwLock<Option<Anime>>>) -> Lists {
        // Top anime
        let top_anime = std::fs::read_to_string("top_anime.json")
            .map(read_ranking)
            .map(|x| {
                x.into_iter()
                    .filter(|x| cache.contains_key(x))
                    .collect::<Vec<_>>()
            })
            .unwrap();

        let top_airing = std::fs::read_to_string("top_airing.json")
            .map(read_ranking)
            .map(|x| {
                x.into_iter()
                    .filter(|x| cache.contains_key(x))
                    .collect::<Vec<_>>()
            })
            .unwrap();

        Lists {
            top_anime,
            top_airing,
        }
    }
}

fn read_ranking(data: String) -> Vec<u32> {
    serde_json::from_str::<Vec<Value>>(&data)
        .map(|x| {
            x.into_iter()
                .filter_map(|x| {
                    x.get("node")
                        .map(|x| x.get("id").map(|x| x.as_u64().map(|x| x as u32)))
                        .flatten()
                        .flatten()
                })
                .collect::<Vec<_>>()
        })
        .unwrap()
}
