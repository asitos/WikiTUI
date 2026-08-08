use crate::feed::profile::FeedProfile;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct FeedItem {
    pub title: String,
    pub snippet: String,
    pub categories: Vec<String>,
    pub is_liked: bool,
}

pub enum SelectionStrategy {
    WeightedCategory,
    TopCategory,
    RandomExploration,
}

pub fn choose_strategy() -> SelectionStrategy {
    let mut rng = rand::thread_rng();
    let roll: u8 = rng.gen_range(0..100);
    if roll < 40 {
        SelectionStrategy::WeightedCategory
    } else if roll < 82 {
        SelectionStrategy::TopCategory
    } else {
        SelectionStrategy::RandomExploration
    }
}

pub fn select_best_item(candidates: Vec<FeedItem>, profile: &FeedProfile) -> Option<FeedItem> {
    if candidates.is_empty() {
        return None;
    }

    let strategy = choose_strategy();
    match strategy {
        SelectionStrategy::RandomExploration => {
            let mut rng = rand::thread_rng();
            let idx = rng.gen_range(0..candidates.len());
            Some(candidates[idx].clone())
        }
        SelectionStrategy::TopCategory | SelectionStrategy::WeightedCategory => {
            let mut best_item = None;
            let mut best_score = i32::MIN;

            for item in &candidates {
                if profile.seen_articles.contains(&item.title) {
                    continue;
                }
                let score = profile.score_for_categories(&item.categories);
                if score > best_score {
                    best_score = score;
                    best_item = Some(item);
                }
            }

            best_item.cloned().or_else(|| candidates.first().cloned())
        }
    }
}
